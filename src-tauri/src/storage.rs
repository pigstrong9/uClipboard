use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::models::{ClipItem, ClipKind, NewItem};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS items (
  id          TEXT PRIMARY KEY,
  kind        TEXT NOT NULL,
  content     TEXT NOT NULL,
  extra       TEXT,
  preview     TEXT NOT NULL,
  hash        TEXT NOT NULL,
  size        INTEGER NOT NULL,
  source_app  TEXT,
  favorite    INTEGER NOT NULL DEFAULT 0,
  pinned      INTEGER NOT NULL DEFAULT 0,
  copy_count  INTEGER NOT NULL DEFAULT 1,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL,
  deleted_at  INTEGER
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_items_hash ON items(hash);
CREATE INDEX IF NOT EXISTS idx_items_order ON items(pinned DESC, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_items_kind ON items(kind, favorite, updated_at DESC);
"#;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        Self::from_conn(conn)
    }

    #[cfg(test)]
    pub fn in_memory() -> rusqlite::Result<Self> {
        Self::from_conn(Connection::open_in_memory()?)
    }

    fn from_conn(conn: Connection) -> rusqlite::Result<Self> {
        let store = Self { conn };
        store.conn.execute_batch(SCHEMA)?;
        Ok(store)
    }

    pub fn insert(&self, item: NewItem) -> rusqlite::Result<ClipItem> {
        if let Some(id) = self
            .conn
            .query_row(
                "SELECT id FROM items WHERE hash = ?1 AND deleted_at IS NULL",
                params![item.hash],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            self.touch(&id)?;
            return self.get(&id);
        }

        let id = Uuid::now_v7().to_string();
        let now = now_ms();
        self.conn.execute(
            "INSERT INTO items
             (id, kind, content, extra, preview, hash, size, source_app, favorite, pinned, copy_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, 1, ?9, ?9)",
            params![
                id,
                item.kind.as_str(),
                item.content,
                item.extra,
                item.preview,
                item.hash,
                item.size,
                item.source_app,
                now,
            ],
        )?;
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<ClipItem> {
        self.conn.query_row(
            "SELECT id, kind, content, extra, preview, size, source_app, favorite, pinned, copy_count, created_at, updated_at
             FROM items WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
            row_to_item,
        )
    }

    pub fn list(&self, tab: &str, query: &str) -> rusqlite::Result<Vec<ClipItem>> {
        let pattern = like_pattern(query);
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content, extra, preview, size, source_app, favorite, pinned, copy_count, created_at, updated_at
             FROM items
             WHERE deleted_at IS NULL
               AND (?1 IN ('all', 'favorite') OR kind = ?1)
               AND (?1 != 'favorite' OR favorite = 1)
               AND (?2 = '' OR content LIKE ?3 ESCAPE '\\' OR preview LIKE ?3 ESCAPE '\\')
             ORDER BY (CASE WHEN ?1 = 'favorite' THEN 0 ELSE pinned END) DESC, updated_at DESC
             LIMIT 500",
        )?;
        let rows = stmt.query_map(params![tab, query, pattern], row_to_item)?;
        rows.collect()
    }

    pub fn touch(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE items SET updated_at = ?1, copy_count = copy_count + 1 WHERE id = ?2",
            params![now_ms(), id],
        )?;
        Ok(())
    }

    pub fn toggle_favorite(&self, id: &str) -> rusqlite::Result<ClipItem> {
        self.conn.execute(
            "UPDATE items SET favorite = CASE favorite WHEN 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        self.get(id)
    }

    pub fn toggle_pin(&self, id: &str) -> rusqlite::Result<ClipItem> {
        self.conn.execute(
            "UPDATE items SET pinned = CASE pinned WHEN 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        self.get(id)
    }

    pub fn delete(&self, id: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear(&self) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM items", [])?;
        Ok(())
    }

    pub fn count(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
    }

    pub fn favorite_count(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM items WHERE favorite = 1", [], |row| {
                row.get(0)
            })
    }

    pub fn image_paths(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT content FROM items WHERE kind = 'image'")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    pub fn prune(&self, limit: usize, retention_days: i64) -> rusqlite::Result<Vec<String>> {
        let limit = limit as i64;
        let days = retention_days.max(0);
        let cutoff = now_ms() - days * 24 * 60 * 60 * 1000;
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content FROM items
             WHERE deleted_at IS NULL
               AND favorite = 0 AND pinned = 0
               AND (
                 (?1 > 0 AND id IN (
                   SELECT id FROM items
                   WHERE deleted_at IS NULL AND favorite = 0 AND pinned = 0
                   ORDER BY updated_at DESC
                   LIMIT -1 OFFSET ?1
                 ))
                 OR (?2 > 0 AND updated_at < ?3)
               )",
        )?;
        let doomed = stmt
            .query_map(params![limit, days, cutoff], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(stmt);

        let mut images = Vec::new();
        for (id, kind, content) in doomed {
            if kind == "image" {
                images.push(content);
            }
            self.conn
                .execute("DELETE FROM items WHERE id = ?1", params![id])?;
        }
        Ok(images)
    }
}

fn row_to_item(row: &Row<'_>) -> rusqlite::Result<ClipItem> {
    let kind_value: String = row.get("kind")?;
    let kind = ClipKind::parse(&kind_value).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(1, "kind".to_string(), rusqlite::types::Type::Text)
    })?;
    Ok(ClipItem {
        id: row.get("id")?,
        kind,
        content: row.get("content")?,
        extra: row.get("extra")?,
        preview: row.get("preview")?,
        size: row.get("size")?,
        source_app: row.get("source_app")?,
        favorite: row.get::<_, i64>("favorite")? != 0,
        pinned: row.get::<_, i64>("pinned")? != 0,
        copy_count: row.get("copy_count")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        thumb_data_url: None,
    })
}

fn like_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_text(content: &str, hash: &str) -> NewItem {
        NewItem {
            kind: ClipKind::Text,
            content: content.to_string(),
            extra: None,
            preview: content.chars().take(40).collect(),
            hash: hash.to_string(),
            size: content.chars().count() as i64,
            source_app: None,
        }
    }

    fn new_kind(kind: ClipKind, content: &str, hash: &str) -> NewItem {
        NewItem {
            kind,
            content: content.to_string(),
            extra: None,
            preview: content.to_string(),
            hash: hash.to_string(),
            size: 1,
            source_app: None,
        }
    }

    #[test]
    fn insert_then_list_orders_by_updated_at() {
        let store = Store::in_memory().unwrap();
        let first = store.insert(new_text("first", "h1")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = store.insert(new_text("second", "h2")).unwrap();
        let items = store.list("all", "").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, second.id);
        assert_eq!(items[1].id, first.id);
    }

    #[test]
    fn same_hash_moves_to_top_without_duplicate() {
        let store = Store::in_memory().unwrap();
        let first = store.insert(new_text("alpha", "h1")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let _ = store.insert(new_text("beta", "h2")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let again = store.insert(new_text("alpha", "h1")).unwrap();
        assert_eq!(again.id, first.id);
        assert_eq!(again.copy_count, 2);
        assert_eq!(store.count().unwrap(), 2);
        assert_eq!(store.list("all", "").unwrap()[0].id, first.id);
    }

    #[test]
    fn favorites_filter_and_pin_order() {
        let store = Store::in_memory().unwrap();
        let a = store.insert(new_text("a", "h1")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = store.insert(new_text("b", "h2")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let c = store.insert(new_text("c", "h3")).unwrap();
        store.toggle_favorite(&b.id).unwrap();
        store.toggle_pin(&a.id).unwrap();

        let all = store.list("all", "").unwrap();
        assert_eq!(all[0].id, a.id);
        let favorites = store.list("favorite", "").unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, b.id);
        assert_eq!(store.list("text", "").unwrap()[0].id, a.id);
        assert_eq!(c.kind, ClipKind::Text);
    }

    #[test]
    fn search_and_kind_filter() {
        let store = Store::in_memory().unwrap();
        store.insert(new_text("hello world", "h1")).unwrap();
        store
            .insert(new_kind(ClipKind::Image, "mock.png", "h2"))
            .unwrap();
        let found = store.list("all", "hello").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].content, "hello world");
        assert_eq!(store.list("image", "").unwrap().len(), 1);
        assert_eq!(store.list("text", "").unwrap().len(), 1);
    }

    #[test]
    fn delete_and_clear() {
        let store = Store::in_memory().unwrap();
        let a = store.insert(new_text("a", "h1")).unwrap();
        store.insert(new_text("b", "h2")).unwrap();
        store.delete(&a.id).unwrap();
        assert_eq!(store.count().unwrap(), 1);
        store.clear().unwrap();
        assert_eq!(store.count().unwrap(), 0);
    }

    #[test]
    fn prune_keeps_favorites_and_pinned() {
        let store = Store::in_memory().unwrap();
        let a = store.insert(new_text("a", "h1")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = store.insert(new_text("b", "h2")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let c = store.insert(new_text("c", "h3")).unwrap();
        store.toggle_pin(&a.id).unwrap();
        store.toggle_favorite(&b.id).unwrap();
        store.prune(1, 0).unwrap();
        assert_eq!(store.count().unwrap(), 3);
        assert_eq!(store.get(&c.id).unwrap().id, c.id);
    }

    #[test]
    fn prune_zero_limit_disables_count_pruning() {
        let store = Store::in_memory().unwrap();
        store.insert(new_text("a", "h1")).unwrap();
        store.insert(new_text("b", "h2")).unwrap();
        store.prune(0, 0).unwrap();
        assert_eq!(store.count().unwrap(), 2);
    }

    #[test]
    fn prune_returns_removed_image_paths() {
        let store = Store::in_memory().unwrap();
        store
            .insert(new_kind(ClipKind::Image, "images/old.png", "h1"))
            .unwrap();
        store
            .conn
            .execute("UPDATE items SET updated_at = 0", [])
            .unwrap();
        let removed = store.prune(0, 1).unwrap();
        assert_eq!(removed, vec!["images/old.png".to_string()]);
        assert_eq!(store.count().unwrap(), 0);
    }
}

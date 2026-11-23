use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::media;
use crate::models::{ClipItem, ClipKind, ClipSummary, NewItem};

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
  updated_at  INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_items_hash ON items(hash);
CREATE INDEX IF NOT EXISTS idx_items_order ON items(pinned DESC, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_items_kind ON items(kind, favorite, updated_at DESC);
"#;

/// Ordered schema history. Version 1 is the original schema; append future upgrades
/// instead of editing an entry that has already shipped.
const MIGRATIONS: &[&str] = &[SCHEMA];

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
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let version: i64 = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        let latest = MIGRATIONS.len() as i64;
        if version >= latest {
            return Ok(());
        }
        // One transaction per startup keeps a partially applied upgrade from leaving the
        // file at a version the code no longer understands.
        let tx = self.conn.unchecked_transaction()?;
        for (index, sql) in MIGRATIONS.iter().enumerate().skip(version.max(0) as usize) {
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", (index + 1) as i64)?;
        }
        tx.commit()
    }

    pub fn insert(&self, item: NewItem) -> rusqlite::Result<ClipItem> {
        if let Some(id) = self
            .conn
            .query_row(
                "SELECT id FROM items WHERE hash = ?1",
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
             FROM items WHERE id = ?1",
            params![id],
            row_to_item,
        )
    }

    #[cfg(test)]
    pub fn list(&self, tab: &str, query: &str, limit: i64) -> rusqlite::Result<Vec<ClipItem>> {
        let pattern = like_pattern(query);
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content, extra, preview, size, source_app, favorite, pinned, copy_count, created_at, updated_at
             FROM items
             WHERE (?1 IN ('all', 'favorite') OR kind = ?1)
               AND (?1 != 'favorite' OR favorite = 1)
               AND (?2 = '' OR content LIKE ?3 ESCAPE '\\' OR preview LIKE ?3 ESCAPE '\\')
             ORDER BY (CASE WHEN ?1 = 'favorite' THEN 0 ELSE pinned END) DESC, updated_at DESC
             LIMIT ?4",
        )?;
        let rows = stmt.query_map(params![tab, query, pattern, limit], row_to_item)?;
        rows.collect()
    }

    /// Lightweight rows for the history list: long text and HTML are never selected, so
    /// a 5 MB clip cannot be shipped to the webview on every refresh.
    pub fn list_summaries(
        &self,
        tab: &str,
        query: &str,
        limit: i64,
        media_dir: &Path,
    ) -> rusqlite::Result<Vec<ClipSummary>> {
        let pattern = like_pattern(query);
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, preview, size, source_app, favorite, pinned, copy_count,
                    created_at, updated_at,
                    CASE WHEN kind = 'text' THEN substr(content, 1, 400) ELSE '' END AS text,
                    CASE WHEN kind = 'text' AND length(content) > 400 THEN 1 ELSE 0 END AS truncated,
                    CASE WHEN kind = 'files' AND length(content) <= 4096 THEN content END AS files_json,
                    CASE WHEN kind = 'image' THEN content END AS image_path,
                    json_extract(extra, '$.width') AS width,
                    json_extract(extra, '$.height') AS height,
                    json_extract(extra, '$.count') AS file_count
             FROM items
             WHERE (?1 IN ('all', 'favorite') OR kind = ?1)
               AND (?1 != 'favorite' OR favorite = 1)
               AND (?2 = '' OR content LIKE ?3 ESCAPE '\\' OR preview LIKE ?3 ESCAPE '\\')
               AND (
                 favorite = 1
                 OR pinned = 1
                 OR id IN (
                   SELECT id FROM items
                   WHERE favorite = 0 AND pinned = 0
                     AND (?1 IN ('all', 'favorite') OR kind = ?1)
                     AND (?2 = '' OR content LIKE ?3 ESCAPE '\\' OR preview LIKE ?3 ESCAPE '\\')
                   ORDER BY updated_at DESC
                   LIMIT ?4
                 )
               )
             ORDER BY (CASE WHEN ?1 = 'favorite' THEN 0 ELSE pinned END) DESC, updated_at DESC
             LIMIT -1",
        )?;
        let rows = stmt.query_map(params![tab, query, pattern, limit], |row| {
            let kind_value: String = row.get("kind")?;
            let kind = ClipKind::parse(&kind_value).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    1,
                    "kind".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let files_json: Option<String> = row.get("files_json")?;
            let files = files_json
                .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                .unwrap_or_default();
            let image_path: Option<String> = row.get("image_path")?;
            let thumb_path = image_path
                .as_deref()
                .and_then(|path| media::thumb_path(media_dir, path));
            Ok(ClipSummary {
                id: row.get("id")?,
                kind,
                preview: row.get("preview")?,
                text: row.get("text")?,
                truncated: row.get::<_, i64>("truncated")? != 0,
                files,
                width: row
                    .get::<_, Option<i64>>("width")?
                    .and_then(|value| u32::try_from(value).ok()),
                height: row
                    .get::<_, Option<i64>>("height")?
                    .and_then(|value| u32::try_from(value).ok()),
                file_count: row.get("file_count")?,
                size: row.get("size")?,
                source_app: row.get("source_app")?,
                favorite: row.get::<_, i64>("favorite")? != 0,
                pinned: row.get::<_, i64>("pinned")? != 0,
                copy_count: row.get("copy_count")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                thumb_path,
            })
        })?;
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
             WHERE favorite = 0 AND pinned = 0
               AND (
                 (?1 > 0 AND id IN (
                   SELECT id FROM items
                   WHERE favorite = 0 AND pinned = 0
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

        if doomed.is_empty() {
            return Ok(Vec::new());
        }

        // One transaction keeps a half-applied prune from orphaning image files, and turns
        // N separate fsyncs into one when the history is long.
        let tx = self.conn.unchecked_transaction()?;
        let mut images = Vec::new();
        for (id, kind, content) in doomed {
            if kind == "image" {
                images.push(content);
            }
            tx.execute("DELETE FROM items WHERE id = ?1", params![id])?;
        }
        tx.commit()?;
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
        thumb_path: None,
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
        let items = store.list("all", "", 500).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, second.id);
        assert_eq!(items[1].id, first.id);
    }

    #[test]
    fn migrates_schema_and_keeps_data_on_reopen() {
        let dir = std::env::temp_dir().join(format!("uclipboard-store-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        {
            let store = Store::open(&path).unwrap();
            let version: i64 = store
                .conn
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .unwrap();
            assert_eq!(version, MIGRATIONS.len() as i64);
            store.insert(new_text("kept", "h1")).unwrap();
        }
        let reopened = Store::open(&path).unwrap();
        assert_eq!(reopened.count().unwrap(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn summaries_avoid_full_payloads_and_resolve_thumbnails() {
        let dir = std::env::temp_dir().join(format!("uclipboard-summary-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(dir.join("images")).unwrap();
        std::fs::write(dir.join("images/pic_thumb.jpg"), b"thumb").unwrap();

        let store = Store::in_memory().unwrap();
        let long = "x".repeat(1_000);
        store.insert(new_text(&long, "h1")).unwrap();
        store
            .insert(new_kind(ClipKind::Image, "images/pic.png", "h2"))
            .unwrap();
        store
            .insert(new_kind(
                ClipKind::Files,
                "[\"C:/tmp/a.txt\",\"C:/tmp/b.txt\"]",
                "h3",
            ))
            .unwrap();

        let summaries = store.list_summaries("all", "", 500, &dir).unwrap();
        let text = summaries
            .iter()
            .find(|row| row.kind == ClipKind::Text)
            .unwrap();
        assert!(text.truncated);
        assert_eq!(text.text.chars().count(), 400);

        let image = summaries
            .iter()
            .find(|row| row.kind == ClipKind::Image)
            .unwrap();
        assert!(image
            .thumb_path
            .as_deref()
            .unwrap_or_default()
            .ends_with("pic_thumb.jpg"));

        let files = summaries
            .iter()
            .find(|row| row.kind == ClipKind::Files)
            .unwrap();
        assert_eq!(files.files, vec!["C:/tmp/a.txt", "C:/tmp/b.txt"]);
        let _ = std::fs::remove_dir_all(dir);
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
        assert_eq!(store.list("all", "", 500).unwrap()[0].id, first.id);
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

        let all = store.list("all", "", 500).unwrap();
        assert_eq!(all[0].id, a.id);
        let favorites = store.list("favorite", "", 500).unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, b.id);
        assert_eq!(store.list("text", "", 500).unwrap()[0].id, a.id);
        assert_eq!(c.kind, ClipKind::Text);
    }

    #[test]
    fn favorite_list_is_not_capped_by_history_limit() {
        let store = Store::in_memory().unwrap();
        for index in 0..5 {
            let item = store
                .insert(new_text(&format!("item {index}"), &format!("h{index}")))
                .unwrap();
            store.toggle_favorite(&item.id).unwrap();
        }

        assert_eq!(store.list("all", "", 2).unwrap().len(), 2);
        assert_eq!(store.list("favorite", "", -1).unwrap().len(), 5);
    }

    #[test]
    fn summaries_keep_pinned_past_the_limit() {
        let store = Store::in_memory().unwrap();
        let mut items = Vec::new();
        for index in 0..5 {
            items.push(
                store
                    .insert(new_text(&format!("item {index}"), &format!("h{index}")))
                    .unwrap(),
            );
        }
        // Pin the newest entries so a limit applied before the exemption would hide the
        // older, non-pinned rows entirely.
        for item in items.iter().rev().take(3) {
            store.toggle_pin(&item.id).unwrap();
        }

        let rows = store.list_summaries("all", "", 2, Path::new(".")).unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows.iter().filter(|row| row.pinned).count(), 3);
        assert_eq!(rows.iter().filter(|row| !row.pinned).count(), 2);
    }

    #[test]
    fn search_and_kind_filter() {
        let store = Store::in_memory().unwrap();
        store.insert(new_text("hello world", "h1")).unwrap();
        store
            .insert(new_kind(ClipKind::Image, "mock.png", "h2"))
            .unwrap();
        let found = store.list("all", "hello", 500).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].content, "hello world");
        assert_eq!(store.list("image", "", 500).unwrap().len(), 1);
        assert_eq!(store.list("text", "", 500).unwrap().len(), 1);
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

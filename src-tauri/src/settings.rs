use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::models::ClipKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub hotkey: String,
    pub screenshot_hotkey: String,
    pub auto_start: bool,
    pub hide_on_blur: bool,
    pub theme: String,
    pub language: String,
    pub history_limit: usize,
    pub retention_days: i64,
    pub record_text: bool,
    pub record_images: bool,
    pub record_files: bool,
    pub min_text_length: usize,
    pub ignored_apps: Vec<String>,
    pub paste_plain_text: bool,
    pub auto_update: bool,
    pub monitor_paused: bool,
    pub max_text_bytes: usize,
    pub max_image_bytes: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Alt+V".to_string(),
            screenshot_hotkey: "Alt+A".to_string(),
            auto_start: !cfg!(debug_assertions),
            hide_on_blur: true,
            theme: "system".to_string(),
            language: "system".to_string(),
            history_limit: 500,
            retention_days: 30,
            record_text: true,
            record_images: true,
            record_files: true,
            min_text_length: 3,
            ignored_apps: Vec::new(),
            paste_plain_text: false,
            auto_update: true,
            monitor_paused: false,
            max_text_bytes: 5 * 1024 * 1024,
            max_image_bytes: 20 * 1024 * 1024,
        }
    }
}

impl Settings {
    pub fn normalized(mut self) -> Self {
        let defaults = Settings::default();
        self.hotkey = self.hotkey.trim().to_string();
        if self.hotkey.is_empty() {
            self.hotkey = defaults.hotkey;
        }
        self.screenshot_hotkey = self.screenshot_hotkey.trim().to_string();
        if self.screenshot_hotkey.is_empty() {
            self.screenshot_hotkey = defaults.screenshot_hotkey;
        }
        if !matches!(self.theme.as_str(), "system" | "light" | "dark") {
            self.theme = defaults.theme;
        }
        if !matches!(self.language.as_str(), "system" | "zh-CN" | "en-US") {
            self.language = defaults.language;
        }
        self.history_limit = self.history_limit.min(10_000);
        self.retention_days = self.retention_days.clamp(0, 3_650);
        self.min_text_length = self.min_text_length.clamp(1, 100);
        self.max_text_bytes = self.max_text_bytes.clamp(1024, 50 * 1024 * 1024);
        self.max_image_bytes = self.max_image_bytes.clamp(1024 * 1024, 100 * 1024 * 1024);

        let mut apps: Vec<String> = Vec::new();
        for app in self.ignored_apps {
            let trimmed = app.trim();
            if trimmed.is_empty() {
                continue;
            }
            if apps
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(trimmed))
            {
                continue;
            }
            apps.push(trimmed.to_string());
        }
        self.ignored_apps = apps;
        self
    }

    pub fn hotkeys_conflict(&self) -> bool {
        self.screenshot_hotkey.eq_ignore_ascii_case(&self.hotkey)
    }

    pub fn allows(&self, kind: ClipKind) -> bool {
        match kind {
            ClipKind::Text => self.record_text,
            ClipKind::Image => self.record_images,
            ClipKind::Files => self.record_files,
        }
    }

    pub fn ignores_app(&self, process: &str) -> bool {
        let name = process
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or(process)
            .trim()
            .to_ascii_lowercase();
        if name.is_empty() {
            return false;
        }
        self.ignored_apps.iter().any(|app| {
            let configured = app
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or(app)
                .trim()
                .to_ascii_lowercase();
            configured == name
        })
    }
}

pub fn load(path: &Path) -> Settings {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return Settings::default().normalized(),
    };
    match serde_json::from_str::<Settings>(&raw) {
        Ok(settings) => settings.normalized(),
        Err(error) => {
            eprintln!("config.json parse failed ({error}); backing up and resetting to defaults");
            let backup = path.with_extension("json.bak");
            let _ = std::fs::write(&backup, &raw);
            Settings::default().normalized()
        }
    }
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let raw = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    // Write beside the target and rename, so a crash mid-write cannot leave a truncated
    // config.json behind (rename replaces the old file atomically on all targets).
    let temp = path.with_extension("json.tmp");
    let write = (|| -> Result<(), String> {
        let mut file = std::fs::File::create(&temp).map_err(|error| error.to_string())?;
        file.write_all(raw.as_bytes())
            .map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())
    })();
    if let Err(error) = write {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    if let Err(error) = std::fs::rename(&temp, path) {
        let _ = std::fs::remove_file(&temp);
        return Err(error.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_bad_values_and_ignored_apps() {
        let settings = Settings {
            hotkey: "  ".to_string(),
            theme: "neon".to_string(),
            language: "fr".to_string(),
            history_limit: 999_999,
            retention_days: -3,
            min_text_length: 0,
            ignored_apps: vec![
                " KeePass.exe ".to_string(),
                "keepass.exe".to_string(),
                String::new(),
                "C:\\Program Files\\1Password\\1Password.exe".to_string(),
            ],
            ..Settings::default()
        }
        .normalized();

        assert_eq!(settings.hotkey, "Alt+V");
        assert_eq!(settings.theme, "system");
        assert_eq!(settings.language, "system");
        assert_eq!(settings.history_limit, 10_000);
        assert_eq!(settings.retention_days, 0);
        assert_eq!(settings.min_text_length, 1);
        assert_eq!(settings.ignored_apps.len(), 2);
    }

    #[test]
    fn defaults_and_normalizes_screenshot_hotkey() {
        assert_eq!(Settings::default().screenshot_hotkey, "Alt+A");
        let settings = Settings {
            screenshot_hotkey: "  ".to_string(),
            ..Settings::default()
        }
        .normalized();
        assert_eq!(settings.screenshot_hotkey, "Alt+A");
    }

    #[test]
    fn detects_conflicting_hotkeys() {
        let settings = Settings {
            hotkey: "Alt+A".to_string(),
            screenshot_hotkey: "alt+a".to_string(),
            ..Settings::default()
        };
        assert!(settings.hotkeys_conflict());
        assert!(!Settings::default().hotkeys_conflict());
    }

    #[test]
    fn matches_ignored_app_by_process_name() {
        let settings = Settings {
            ignored_apps: vec!["KeePassXC.exe".to_string()],
            ..Settings::default()
        }
        .normalized();
        assert!(settings.ignores_app("C:\\Program Files\\KeePassXC\\KeePassXC.exe"));
        assert!(!settings.ignores_app("notepad.exe"));
    }

    #[test]
    fn saves_and_loads_config() {
        let dir =
            std::env::temp_dir().join(format!("uclipboard-settings-{}", uuid::Uuid::now_v7()));
        let path = dir.join("config.json");
        let settings = Settings {
            hotkey: "Ctrl+Shift+V".to_string(),
            history_limit: 42,
            ..Settings::default()
        };
        save(&path, &settings).unwrap();
        let loaded = load(&path);
        assert_eq!(loaded.hotkey, "Ctrl+Shift+V");
        assert_eq!(loaded.history_limit, 42);
        let _ = std::fs::remove_dir_all(dir);
    }
}

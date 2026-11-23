use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClipKind {
    Text,
    Image,
    Files,
}

impl ClipKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClipKind::Text => "text",
            ClipKind::Image => "image",
            ClipKind::Files => "files",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(ClipKind::Text),
            "image" => Some(ClipKind::Image),
            "files" => Some(ClipKind::Files),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: String,
    pub kind: ClipKind,
    pub content: String,
    pub extra: Option<String>,
    pub preview: String,
    pub size: i64,
    pub source_app: Option<String>,
    pub favorite: bool,
    pub pinned: bool,
    pub copy_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumb_path: Option<String>,
}

/// The lightweight shape the history list renders. It deliberately omits the full
/// `content`/`extra` payloads so a long history cannot ship megabytes of text or HTML
/// to the webview on every refresh.
#[derive(Debug, Clone, Serialize)]
pub struct ClipSummary {
    pub id: String,
    pub kind: ClipKind,
    pub preview: String,
    pub text: String,
    pub truncated: bool,
    pub files: Vec<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_count: Option<i64>,
    pub size: i64,
    pub source_app: Option<String>,
    pub favorite: bool,
    pub pinned: bool,
    pub copy_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumb_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewItem {
    pub kind: ClipKind,
    pub content: String,
    pub extra: Option<String>,
    pub preview: String,
    pub hash: String,
    pub size: i64,
    pub source_app: Option<String>,
}

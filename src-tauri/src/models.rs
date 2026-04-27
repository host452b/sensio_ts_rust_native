use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppBootstrap {
    pub library_path: String,
    pub database_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DocumentRecord {
    pub id: String,
    pub title: String,
    pub original_path: String,
    pub storage_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub imported_at: String,
    pub updated_at: String,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DocumentQuery {
    pub search: Option<String>,
    pub tag_ids: Vec<String>,
    pub sort_key: SortKey,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DocumentList {
    pub items: Vec<DocumentRecord>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchHit {
    pub document_id: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HighlightRect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HighlightRecord {
    pub id: String,
    pub document_id: String,
    pub page_index: i64,
    pub rects: Vec<HighlightRect>,
    pub color: String,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HighlightInput {
    pub document_id: String,
    pub page_index: i64,
    pub rects: Vec<HighlightRect>,
    pub color: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SortKey {
    Recent,
    Title,
    Size,
}

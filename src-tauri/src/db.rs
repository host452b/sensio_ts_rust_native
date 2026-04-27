use std::path::{Path, PathBuf};

use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension};
use uuid::Uuid;

use crate::models::{
    AppSetting, DocumentList, DocumentQuery, DocumentRecord, HighlightInput, HighlightRecord,
    SearchHit, SortKey,
};

const DATABASE_FILE: &str = "AppState.db";

pub fn database_path(library_dir: &Path) -> PathBuf {
    library_dir.join(DATABASE_FILE)
}

pub fn documents_dir(library_dir: &Path) -> PathBuf {
    library_dir.join("documents")
}

pub fn initialize_library(library_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(library_dir)
        .map_err(|error| format!("无法创建资料库目录 {}: {error}", library_dir.display()))?;
    std::fs::create_dir_all(documents_dir(library_dir)).map_err(|error| {
        format!(
            "无法创建文档目录 {}: {error}",
            documents_dir(library_dir).display()
        )
    })?;

    let conn = open_connection(library_dir)?;
    create_schema(&conn)?;
    seed_settings(&conn)?;
    Ok(())
}

pub fn open_connection(library_dir: &Path) -> Result<Connection, String> {
    let conn = Connection::open(database_path(library_dir))
        .map_err(|error| format!("无法打开 SQLite 数据库: {error}"))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| format!("无法启用 foreign_keys: {error}"))?;
    Ok(conn)
}

pub fn insert_document(conn: &Connection, record: &DocumentRecord) -> Result<(), String> {
    conn.execute(
        "INSERT INTO documents (
            id, title, original_path, storage_path, file_name, file_size,
            imported_at, updated_at, thumbnail_path
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            record.id,
            record.title,
            record.original_path,
            record.storage_path,
            record.file_name,
            record.file_size,
            record.imported_at,
            record.updated_at,
            record.thumbnail_path,
        ],
    )
    .map_err(|error| format!("无法写入文档记录: {error}"))?;
    Ok(())
}

pub fn get_document(conn: &Connection, document_id: &str) -> Result<DocumentRecord, String> {
    conn.query_row(
        "SELECT id, title, original_path, storage_path, file_name, file_size,
            imported_at, updated_at, thumbnail_path
         FROM documents
         WHERE id = ?1",
        params![document_id],
        read_document_row,
    )
    .map_err(|error| format!("找不到文档 {document_id}: {error}"))
}

pub fn list_documents(conn: &Connection, query: DocumentQuery) -> Result<DocumentList, String> {
    let limit = query.limit.clamp(1, 200);
    let offset = query.offset.max(0);
    let filters = build_filters(&query);
    let order_by = match query.sort_key {
        SortKey::Recent => "d.updated_at DESC, d.id ASC",
        SortKey::Title => "d.title COLLATE NOCASE ASC, d.id ASC",
        SortKey::Size => "d.file_size DESC, d.id ASC",
    };

    let count_sql = format!("SELECT COUNT(*) FROM documents d {}", filters.where_clause);
    let total: i64 = conn
        .query_row(&count_sql, params_from_iter(filters.params.iter()), |row| {
            row.get(0)
        })
        .map_err(|error| format!("无法统计文档: {error}"))?;

    let mut list_params = filters.params.clone();
    list_params.push(Value::Integer(limit));
    list_params.push(Value::Integer(offset));
    let list_sql = format!(
        "SELECT d.id, d.title, d.original_path, d.storage_path, d.file_name, d.file_size,
            d.imported_at, d.updated_at, d.thumbnail_path
         FROM documents d
         {}
         ORDER BY {}
         LIMIT ? OFFSET ?",
        filters.where_clause, order_by
    );

    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|error| format!("无法准备文档查询: {error}"))?;
    let items = stmt
        .query_map(params_from_iter(list_params.iter()), read_document_row)
        .map_err(|error| format!("无法查询文档: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法读取文档记录: {error}"))?;

    Ok(DocumentList { items, total })
}

pub fn search_documents(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchHit>, String> {
    let normalized = query.trim();
    if normalized.is_empty() {
        return Ok(Vec::new());
    }

    let limit = limit.clamp(1, 100);
    let fts_query = fts_phrase(normalized);
    let mut hits = query_search_table(
        conn,
        "documents_fts",
        "snippet(documents_fts, 1, '<mark>', '</mark>', '...', 16)",
        &fts_query,
        limit,
    )?;

    if hits.len() < limit as usize {
        let remaining = limit - hits.len() as i64;
        let trigram_hits = query_search_table(
            conn,
            "documents_trigram",
            "snippet(documents_trigram, 1, '<mark>', '</mark>', '...', 16)",
            &fts_query,
            remaining,
        )?;
        for hit in trigram_hits {
            if !hits
                .iter()
                .any(|existing| existing.document_id == hit.document_id)
            {
                hits.push(hit);
            }
        }
    }

    Ok(hits)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<AppSetting>, String> {
    conn.query_row(
        "SELECT key, value, updated_at FROM app_settings WHERE key = ?1",
        params![key],
        |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(|error| format!("无法读取设置 {key}: {error}"))
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<AppSetting, String> {
    let updated_at = now_rfc3339();
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, updated_at],
    )
    .map_err(|error| format!("无法保存设置 {key}: {error}"))?;

    Ok(AppSetting {
        key: key.to_string(),
        value: value.to_string(),
        updated_at,
    })
}

pub fn list_highlights(
    conn: &Connection,
    document_id: &str,
    page_index: Option<i64>,
) -> Result<Vec<HighlightRecord>, String> {
    let (sql, values) = match page_index {
        Some(page_index) => (
            "SELECT id, document_id, page_index, rects_json, color, note, created_at
             FROM highlights
             WHERE document_id = ?1 AND page_index = ?2
             ORDER BY page_index ASC, created_at ASC"
                .to_string(),
            vec![
                Value::Text(document_id.to_string()),
                Value::Integer(page_index),
            ],
        ),
        None => (
            "SELECT id, document_id, page_index, rects_json, color, note, created_at
             FROM highlights
             WHERE document_id = ?1
             ORDER BY page_index ASC, created_at ASC"
                .to_string(),
            vec![Value::Text(document_id.to_string())],
        ),
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|error| format!("无法准备高亮查询: {error}"))?;
    let records = stmt
        .query_map(params_from_iter(values.iter()), read_highlight_row)
        .map_err(|error| format!("无法查询高亮: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法读取高亮记录: {error}"))?;
    Ok(records)
}

pub fn insert_highlight(
    conn: &Connection,
    input: HighlightInput,
) -> Result<HighlightRecord, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = now_rfc3339();
    let rects_json = serde_json::to_string(&input.rects)
        .map_err(|error| format!("无法序列化高亮区域: {error}"))?;

    conn.execute(
        "INSERT INTO highlights (
            id, document_id, page_index, rects_json, color, note, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.document_id,
            input.page_index,
            rects_json,
            input.color,
            input.note,
            created_at,
        ],
    )
    .map_err(|error| format!("无法保存高亮: {error}"))?;

    Ok(HighlightRecord {
        id,
        document_id: input.document_id,
        page_index: input.page_index,
        rects: input.rects,
        color: input.color,
        note: input.note,
        created_at,
    })
}

pub fn delete_highlight(conn: &Connection, highlight_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM highlights WHERE id = ?1",
        params![highlight_id],
    )
    .map_err(|error| format!("无法删除高亮 {highlight_id}: {error}"))?;
    Ok(())
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn create_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            original_path TEXT NOT NULL,
            storage_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            imported_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            thumbnail_path TEXT
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            document_id UNINDEXED,
            title,
            file_name,
            original_path,
            tokenize = 'unicode61'
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS documents_trigram USING fts5(
            document_id UNINDEXED,
            title,
            file_name,
            tokenize = 'trigram'
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL COLLATE NOCASE UNIQUE,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS document_tags (
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (document_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS highlights (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            page_index INTEGER NOT NULL,
            rects_json TEXT NOT NULL,
            color TEXT NOT NULL,
            note TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_documents_updated_at ON documents(updated_at);
        CREATE INDEX IF NOT EXISTS idx_documents_title ON documents(title COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_document_tags_tag ON document_tags(tag_id, document_id);
        CREATE INDEX IF NOT EXISTS idx_highlights_document_page ON highlights(document_id, page_index);

        CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
            INSERT INTO documents_fts(document_id, title, file_name, original_path)
            VALUES (new.id, new.title, new.file_name, new.original_path);
            INSERT INTO documents_trigram(document_id, title, file_name)
            VALUES (new.id, new.title, new.file_name);
        END;

        CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
            DELETE FROM documents_fts WHERE document_id = old.id;
            DELETE FROM documents_trigram WHERE document_id = old.id;
        END;

        CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE OF title, file_name, original_path ON documents BEGIN
            DELETE FROM documents_fts WHERE document_id = old.id;
            DELETE FROM documents_trigram WHERE document_id = old.id;
            INSERT INTO documents_fts(document_id, title, file_name, original_path)
            VALUES (new.id, new.title, new.file_name, new.original_path);
            INSERT INTO documents_trigram(document_id, title, file_name)
            VALUES (new.id, new.title, new.file_name);
        END;
        ",
    )
    .map_err(|error| format!("无法创建 SQLite schema: {error}"))?;
    Ok(())
}

fn seed_settings(conn: &Connection) -> Result<(), String> {
    let now = now_rfc3339();
    for (key, value) in [("layoutMode", "grid"), ("sortKey", "recent")] {
        conn.execute(
            "INSERT OR IGNORE INTO app_settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key, value, now],
        )
        .map_err(|error| format!("无法初始化设置 {key}: {error}"))?;
    }
    Ok(())
}

struct QueryFilters {
    where_clause: String,
    params: Vec<Value>,
}

fn build_filters(query: &DocumentQuery) -> QueryFilters {
    let mut filters = Vec::new();
    let mut params = Vec::new();

    if let Some(search) = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let like = format!("%{}%", search.replace('%', "\\%").replace('_', "\\_"));
        let fts_query = fts_phrase(search);
        filters.push(
            "(d.title LIKE ? ESCAPE '\\'
              OR d.file_name LIKE ? ESCAPE '\\'
              OR d.original_path LIKE ? ESCAPE '\\'
              OR d.id IN (SELECT document_id FROM documents_fts WHERE documents_fts MATCH ?)
              OR d.id IN (SELECT document_id FROM documents_trigram WHERE documents_trigram MATCH ?))"
                .to_string(),
        );
        params.push(Value::Text(like.clone()));
        params.push(Value::Text(like.clone()));
        params.push(Value::Text(like));
        params.push(Value::Text(fts_query.clone()));
        params.push(Value::Text(fts_query));
    }

    for tag_id in &query.tag_ids {
        filters.push(
            "EXISTS (
                SELECT 1 FROM document_tags dt
                WHERE dt.document_id = d.id AND dt.tag_id = ?
            )"
            .to_string(),
        );
        params.push(Value::Text(tag_id.clone()));
    }

    let where_clause = if filters.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", filters.join(" AND "))
    };

    QueryFilters {
        where_clause,
        params,
    }
}

fn query_search_table(
    conn: &Connection,
    table_name: &str,
    snippet_expression: &str,
    fts_query: &str,
    limit: i64,
) -> Result<Vec<SearchHit>, String> {
    let sql = format!(
        "SELECT d.id, d.title, {snippet_expression}, bm25({table_name}) AS score
         FROM {table_name}
         JOIN documents d ON d.id = {table_name}.document_id
         WHERE {table_name} MATCH ?1
         ORDER BY score ASC
         LIMIT ?2"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|error| format!("无法准备搜索查询: {error}"))?;

    let hits = stmt
        .query_map(params![fts_query, limit], |row| {
            Ok(SearchHit {
                document_id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                score: row.get(3)?,
            })
        })
        .map_err(|error| format!("无法执行搜索: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法读取搜索结果: {error}"))?;
    Ok(hits)
}

fn fts_phrase(input: &str) -> String {
    format!("\"{}\"", input.replace('"', "\"\""))
}

fn read_document_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocumentRecord> {
    Ok(DocumentRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        original_path: row.get(2)?,
        storage_path: row.get(3)?,
        file_name: row.get(4)?,
        file_size: row.get(5)?,
        imported_at: row.get(6)?,
        updated_at: row.get(7)?,
        thumbnail_path: row.get(8)?,
    })
}

fn read_highlight_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HighlightRecord> {
    let rects_json: String = row.get(3)?;
    let rects = serde_json::from_str(&rects_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(HighlightRecord {
        id: row.get(0)?,
        document_id: row.get(1)?,
        page_index: row.get(2)?,
        rects,
        color: row.get(4)?,
        note: row.get(5)?,
        created_at: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rusqlite::params;
    use uuid::Uuid;

    use super::*;
    use crate::models::{HighlightRect, SortKey};

    #[test]
    fn initializes_library_and_lists_documents_with_unique_tag_filter() {
        let library_dir = create_temp_library();
        initialize_library(&library_dir).expect("initialize library");
        let conn = open_connection(&library_dir).expect("open connection");
        let document = sample_document("机器学习基础", &library_dir);
        insert_document(&conn, &document).expect("insert document");
        conn.execute(
            "INSERT INTO tags (id, name, created_at) VALUES (?1, ?2, ?3)",
            params!["tag-a", "AI", now_rfc3339()],
        )
        .expect("insert tag");
        conn.execute(
            "INSERT INTO document_tags (document_id, tag_id) VALUES (?1, ?2)",
            params![document.id, "tag-a"],
        )
        .expect("insert document tag");

        let list = list_documents(
            &conn,
            DocumentQuery {
                search: Some("机器".to_string()),
                tag_ids: vec!["tag-a".to_string()],
                sort_key: SortKey::Recent,
                offset: 0,
                limit: 20,
            },
        )
        .expect("list documents");

        assert_eq!(list.total, 1);
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].title, "机器学习基础");
    }

    #[test]
    fn searches_documents_with_trigram_substring_fallback() {
        let library_dir = create_temp_library();
        initialize_library(&library_dir).expect("initialize library");
        let conn = open_connection(&library_dir).expect("open connection");
        let document = sample_document("Local first PDF archive", &library_dir);
        insert_document(&conn, &document).expect("insert document");

        let hits = search_documents(&conn, "first", 10).expect("search documents");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document_id, document.id);
    }

    #[test]
    fn settings_and_highlights_round_trip() {
        let library_dir = create_temp_library();
        initialize_library(&library_dir).expect("initialize library");
        let conn = open_connection(&library_dir).expect("open connection");
        let document = sample_document("Annotated", &library_dir);
        insert_document(&conn, &document).expect("insert document");

        let setting = set_setting(&conn, "layoutMode", "list").expect("set setting");
        assert_eq!(setting.value, "list");
        assert_eq!(
            get_setting(&conn, "layoutMode")
                .expect("get setting")
                .expect("setting exists")
                .value,
            "list"
        );

        let highlight = insert_highlight(
            &conn,
            HighlightInput {
                document_id: document.id.clone(),
                page_index: 2,
                rects: vec![HighlightRect {
                    left: 0.1,
                    top: 0.2,
                    width: 0.3,
                    height: 0.04,
                }],
                color: "#f6c453".to_string(),
                note: Some("important".to_string()),
            },
        )
        .expect("insert highlight");

        let highlights = list_highlights(&conn, &document.id, Some(2)).expect("list highlights");

        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].id, highlight.id);
        assert_eq!(highlights[0].rects[0].left, 0.1);
        assert_eq!(highlights[0].note.as_deref(), Some("important"));
    }

    fn create_temp_library() -> PathBuf {
        let library_dir = std::env::temp_dir().join(format!("sensio-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&library_dir).expect("create temp library");
        library_dir
    }

    fn sample_document(title: &str, library_dir: &std::path::Path) -> DocumentRecord {
        let id = Uuid::new_v4().to_string();
        let path = documents_dir(library_dir).join(format!("{id}.pdf"));
        DocumentRecord {
            id,
            title: title.to_string(),
            original_path: path.to_string_lossy().into_owned(),
            storage_path: path.to_string_lossy().into_owned(),
            file_name: format!("{title}.pdf"),
            file_size: 42,
            imported_at: now_rfc3339(),
            updated_at: now_rfc3339(),
            thumbnail_path: None,
        }
    }
}

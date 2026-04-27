use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use crate::{
    db,
    models::{
        AppBootstrap, AppSetting, DocumentList, DocumentQuery, DocumentRecord, HighlightInput,
        HighlightRecord, SearchHit,
    },
    state::AppState,
    storage,
};

#[tauri::command]
#[specta::specta]
pub async fn init_app(app: AppHandle, state: State<'_, AppState>) -> Result<AppBootstrap, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录: {error}"))?;

    let (library_dir, database_path) = tokio::task::spawn_blocking(move || {
        let library_dir = storage::load_or_create_library_dir(&app_data_dir)?;
        db::initialize_library(&library_dir)?;
        let database_path = db::database_path(&library_dir);
        Ok::<_, String>((library_dir, database_path))
    })
    .await
    .map_err(|error| format!("初始化任务失败: {error}"))??;

    state.configure_library(library_dir.clone()).await;

    Ok(AppBootstrap {
        library_path: path_to_string(&library_dir),
        database_path: path_to_string(&database_path),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn list_documents(
    state: State<'_, AppState>,
    query: DocumentQuery,
) -> Result<DocumentList, String> {
    run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        db::list_documents(&conn, query)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn import_document(
    app: AppHandle,
    state: State<'_, AppState>,
    source_path: String,
) -> Result<DocumentRecord, String> {
    state.ensure_mutations_allowed().await?;
    let record = run_with_library(state, move |library_dir| {
        import_document_inner(&library_dir, &source_path)
    })
    .await?;
    app.emit("library_updated", record.id.clone())
        .map_err(|error| format!("无法发送 library_updated 事件: {error}"))?;
    Ok(record)
}

#[tauri::command]
#[specta::specta]
pub async fn read_pdf_bytes(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<u8>, String> {
    run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        let document = db::get_document(&conn, &document_id)?;
        fs::read(&document.storage_path)
            .map_err(|error| format!("无法读取 PDF {}: {error}", document.storage_path))
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn search_documents(
    state: State<'_, AppState>,
    query: String,
    limit: i64,
) -> Result<Vec<SearchHit>, String> {
    run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        db::search_documents(&conn, &query, limit)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn list_highlights(
    state: State<'_, AppState>,
    document_id: String,
    page_index: Option<i64>,
) -> Result<Vec<HighlightRecord>, String> {
    run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        db::list_highlights(&conn, &document_id, page_index)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn add_highlight(
    app: AppHandle,
    state: State<'_, AppState>,
    input: HighlightInput,
) -> Result<HighlightRecord, String> {
    state.ensure_mutations_allowed().await?;
    let record = run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        db::insert_highlight(&conn, input)
    })
    .await?;
    app.emit("library_updated", record.document_id.clone())
        .map_err(|error| format!("无法发送 library_updated 事件: {error}"))?;
    Ok(record)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_highlight(
    app: AppHandle,
    state: State<'_, AppState>,
    highlight_id: String,
) -> Result<(), String> {
    state.ensure_mutations_allowed().await?;
    run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        db::delete_highlight(&conn, &highlight_id)
    })
    .await?;
    app.emit("library_updated", "highlight_deleted")
        .map_err(|error| format!("无法发送 library_updated 事件: {error}"))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<AppSetting>, String> {
    run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        db::get_setting(&conn, &key)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn set_setting(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<AppSetting, String> {
    state.ensure_mutations_allowed().await?;
    let setting = run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        db::set_setting(&conn, &key, &value)
    })
    .await?;
    app.emit("settings_updated", setting.key.clone())
        .map_err(|error| format!("无法发送 settings_updated 事件: {error}"))?;
    Ok(setting)
}

#[tauri::command]
#[specta::specta]
pub async fn export_pdf_copy(
    state: State<'_, AppState>,
    document_id: String,
    destination_path: String,
) -> Result<String, String> {
    run_with_library(state, move |library_dir| {
        let conn = db::open_connection(&library_dir)?;
        let document = db::get_document(&conn, &document_id)?;
        let source_path = PathBuf::from(document.storage_path);
        let destination_path = PathBuf::from(destination_path.trim());

        if destination_path.as_os_str().is_empty() {
            return Err("导出路径不能为空".to_string());
        }
        if source_path == destination_path {
            return Err("导出副本不能覆盖资料库内的 PDF".to_string());
        }
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("无法创建导出目录 {}: {error}", parent.display()))?;
        }

        let mut pdf = lopdf::Document::load(&source_path)
            .map_err(|error| format!("无法读取 PDF 副本 {}: {error}", source_path.display()))?;
        pdf.save(&destination_path)
            .map_err(|error| format!("无法写入导出 PDF {}: {error}", destination_path.display()))?;
        Ok(path_to_string(&destination_path))
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn open_document_window(
    app: AppHandle,
    state: State<'_, AppState>,
    document_id: String,
) -> Result<(), String> {
    let label = format!("doc-{}", compact_document_label(&document_id));
    if let Some(window) = app.get_webview_window(&label) {
        window
            .set_focus()
            .map_err(|error| format!("无法聚焦阅读窗口: {error}"))?;
        return Ok(());
    }

    let title = {
        let lookup_document_id = document_id.clone();
        run_with_library(state, move |library_dir| {
            let conn = db::open_connection(&library_dir)?;
            db::get_document(&conn, &lookup_document_id).map(|document| document.title)
        })
        .await?
    };

    let url = format!("index.html#/reader/{document_id}");
    WebviewWindowBuilder::new(&app, label, WebviewUrl::App(url.into()))
        .title(format!("Sensio - {title}"))
        .inner_size(1120.0, 820.0)
        .min_inner_size(820.0, 600.0)
        .build()
        .map_err(|error| format!("无法创建阅读窗口: {error}"))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn pick_pdf_file(app: AppHandle) -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(move || {
        Ok(app
            .dialog()
            .file()
            .set_title("选择 PDF")
            .add_filter("PDF", &["pdf"])
            .blocking_pick_file()
            .map(|path| path.to_string()))
    })
    .await
    .map_err(|error| format!("打开文件选择器失败: {error}"))?
}

#[tauri::command]
#[specta::specta]
pub async fn pick_export_path(
    app: AppHandle,
    suggested_file_name: Option<String>,
) -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(move || {
        let mut dialog = app
            .dialog()
            .file()
            .set_title("导出 PDF 副本")
            .add_filter("PDF", &["pdf"])
            .set_can_create_directories(true);

        if let Some(file_name) = suggested_file_name
            .as_deref()
            .map(storage::sanitize_pdf_name)
            .filter(|value| !value.trim().is_empty())
        {
            dialog = dialog.set_file_name(ensure_pdf_extension(&file_name));
        }

        Ok(dialog.blocking_save_file().map(|path| path.to_string()))
    })
    .await
    .map_err(|error| format!("打开导出位置选择器失败: {error}"))?
}

async fn run_with_library<T, F>(state: State<'_, AppState>, job: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(PathBuf) -> Result<T, String> + Send + 'static,
{
    let library_dir = state.library_dir().await?;
    tokio::task::spawn_blocking(move || job(library_dir))
        .await
        .map_err(|error| format!("后台任务失败: {error}"))?
}

fn import_document_inner(library_dir: &Path, source_path: &str) -> Result<DocumentRecord, String> {
    let source_path = PathBuf::from(source_path.trim());
    if source_path.as_os_str().is_empty() {
        return Err("导入路径不能为空".to_string());
    }

    let canonical_source = source_path
        .canonicalize()
        .map_err(|error| format!("无法读取导入文件 {}: {error}", source_path.display()))?;
    if canonical_source
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| !extension.eq_ignore_ascii_case("pdf"))
        .unwrap_or(true)
    {
        return Err("目前只支持导入 PDF 文件".to_string());
    }

    let metadata = fs::metadata(&canonical_source)
        .map_err(|error| format!("无法读取文件信息 {}: {error}", canonical_source.display()))?;
    if !metadata.is_file() {
        return Err("导入路径不是文件".to_string());
    }

    let file_name = canonical_source
        .file_name()
        .and_then(|value| value.to_str())
        .map(storage::sanitize_pdf_name)
        .ok_or_else(|| "无法识别文件名".to_string())?;
    let title = canonical_source
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| file_name.clone());

    let id = Uuid::new_v4().to_string();
    let storage_path = db::documents_dir(library_dir).join(format!("{id}.pdf"));
    fs::copy(&canonical_source, &storage_path)
        .map_err(|error| format!("无法复制 PDF 到资料库 {}: {error}", storage_path.display()))?;

    let now = db::now_rfc3339();
    let record = DocumentRecord {
        id,
        title,
        original_path: path_to_string(&canonical_source),
        storage_path: path_to_string(&storage_path),
        file_name,
        file_size: metadata.len() as i64,
        imported_at: now.clone(),
        updated_at: now,
        thumbnail_path: None,
    };

    let conn = db::open_connection(library_dir)?;
    db::insert_document(&conn, &record)?;
    Ok(record)
}

fn compact_document_label(document_id: &str) -> String {
    let compact: String = document_id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(16)
        .collect();

    if compact.is_empty() {
        "unknown".to_string()
    } else {
        compact
    }
}

fn ensure_pdf_extension(file_name: &str) -> String {
    if file_name.to_ascii_lowercase().ends_with(".pdf") {
        file_name.to_string()
    } else {
        format!("{file_name}.pdf")
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

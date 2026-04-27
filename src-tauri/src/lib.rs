mod commands;
mod db;
mod models;
mod state;
mod storage;

use commands::{
    add_highlight, delete_highlight, export_pdf_copy, get_setting, import_document, init_app,
    list_documents, list_highlights, open_document_window, pick_export_path, pick_pdf_file,
    read_pdf_bytes, search_documents, set_setting,
};
#[cfg(any(debug_assertions, test))]
use specta_typescript::{BigIntExportBehavior, Typescript};
use state::AppState;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = specta_builder();

    #[cfg(debug_assertions)]
    specta_builder
        .export(typescript_exporter(), "../src/bindings.ts")
        .expect("failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Sensio");
}

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        init_app,
        list_documents,
        import_document,
        read_pdf_bytes,
        search_documents,
        list_highlights,
        add_highlight,
        delete_highlight,
        get_setting,
        set_setting,
        export_pdf_copy,
        open_document_window,
        pick_pdf_file,
        pick_export_path,
    ])
}

#[cfg(any(debug_assertions, test))]
fn typescript_exporter() -> Typescript {
    Typescript::default().bigint(BigIntExportBehavior::Number)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn exports_tauri_specta_command_bindings() {
        let path = std::env::temp_dir().join(format!("sensio-bindings-{}.ts", Uuid::new_v4()));
        specta_builder()
            .export(typescript_exporter(), &path)
            .expect("export bindings");
        let contents = std::fs::read_to_string(&path).expect("read bindings");
        std::fs::remove_file(&path).expect("remove bindings");

        assert!(contents.contains("export const commands"));
        assert!(contents.contains("listDocuments"));
        assert!(contents.contains("openDocumentWindow"));
        assert!(contents.contains("pickPdfFile"));
    }
}

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct LibraryBootstrapFile {
    library_path: String,
}

pub fn bootstrap_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("library.json")
}

pub fn load_or_create_library_dir(app_data_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(app_data_dir)
        .map_err(|error| format!("无法创建应用数据目录 {}: {error}", app_data_dir.display()))?;

    let bootstrap_path = bootstrap_path(app_data_dir);
    if bootstrap_path.exists() {
        let contents = fs::read_to_string(&bootstrap_path)
            .map_err(|error| format!("无法读取 {}: {error}", bootstrap_path.display()))?;
        let bootstrap: LibraryBootstrapFile = serde_json::from_str(&contents)
            .map_err(|error| format!("library.json 格式错误: {error}"))?;
        return Ok(PathBuf::from(bootstrap.library_path));
    }

    let library_dir = app_data_dir.join("library");
    let bootstrap = LibraryBootstrapFile {
        library_path: library_dir.to_string_lossy().into_owned(),
    };
    let contents = serde_json::to_string_pretty(&bootstrap)
        .map_err(|error| format!("无法序列化 library.json: {error}"))?;
    fs::write(&bootstrap_path, contents)
        .map_err(|error| format!("无法写入 {}: {error}", bootstrap_path.display()))?;
    Ok(library_dir)
}

pub fn sanitize_pdf_name(file_name: &str) -> String {
    file_name
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect()
}

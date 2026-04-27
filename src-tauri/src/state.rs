use std::path::PathBuf;

use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct AppState {
    inner: Mutex<RuntimeState>,
}

#[derive(Debug, Default)]
struct RuntimeState {
    library_dir: Option<PathBuf>,
    mutations_frozen: bool,
}

impl AppState {
    pub async fn configure_library(&self, library_dir: PathBuf) {
        let mut inner = self.inner.lock().await;
        inner.library_dir = Some(library_dir);
    }

    pub async fn library_dir(&self) -> Result<PathBuf, String> {
        let inner = self.inner.lock().await;
        inner
            .library_dir
            .clone()
            .ok_or_else(|| "资料库尚未初始化".to_string())
    }

    pub async fn ensure_mutations_allowed(&self) -> Result<(), String> {
        let inner = self.inner.lock().await;
        if inner.mutations_frozen {
            Err("资料库迁移中，暂时不能修改".to_string())
        } else {
            Ok(())
        }
    }
}

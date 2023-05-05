use background_service::error::BackgroundServiceErrors;

use crate::state;

pub struct WorkerGuard;

impl WorkerGuard {
    pub async fn stop(&mut self) -> Result<(), BackgroundServiceErrors> {
        stop().await
    }
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                stop().await.ok();
            });
        }
    }
}

async fn stop() -> Result<(), BackgroundServiceErrors> {
    if let Some(handle) = state::HANDLE.get() {
        let mut handle = handle.lock().await;
        if let Some(handle) = handle.take() {
            return handle.cancel().await;
        }
    }
    Ok(())
}

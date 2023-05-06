use background_service::BackgroundServiceManager;
use once_cell::sync::OnceCell;
use std::sync::{atomic::AtomicBool, RwLock};
use tokio::sync::Mutex;

pub(crate) static IS_INITIALIZED: RwLock<bool> = RwLock::new(false);
pub(crate) static IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static HANDLE: OnceCell<Mutex<Option<BackgroundServiceManager>>> = OnceCell::new();

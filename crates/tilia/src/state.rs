use std::sync::atomic::AtomicBool;
use std::sync::{OnceLock, RwLock};

use background_service::BackgroundServiceManager;
use tokio::sync::Mutex;

pub(crate) static IS_INITIALIZED: RwLock<bool> = RwLock::new(false);
pub(crate) static IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static HANDLE: OnceLock<Mutex<Option<BackgroundServiceManager>>> = OnceLock::new();

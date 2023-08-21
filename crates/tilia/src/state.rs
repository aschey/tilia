use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

use background_service::BackgroundServiceManager;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

pub(crate) static IS_INITIALIZED: RwLock<bool> = RwLock::new(false);
pub(crate) static IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static HANDLE: OnceCell<Mutex<Option<BackgroundServiceManager>>> = OnceCell::new();

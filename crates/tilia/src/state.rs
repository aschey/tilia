use background_service::BackgroundServiceManager;
use once_cell::sync::OnceCell;
use std::sync::{atomic::AtomicBool, RwLock};
use tokio::sync::{broadcast, Mutex};

pub(crate) static IS_INITIALIZED: RwLock<bool> = RwLock::new(false);
pub(crate) static IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static SENDER: OnceCell<broadcast::Sender<Vec<u8>>> = OnceCell::new();
pub(crate) static HANDLE: OnceCell<Mutex<Option<BackgroundServiceManager>>> = OnceCell::new();

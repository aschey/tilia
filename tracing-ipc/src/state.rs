use crate::command::Command;
use once_cell::sync::OnceCell;
use std::sync::{atomic::AtomicBool, RwLock};

pub(crate) static IS_INITIALIZED: RwLock<bool> = RwLock::new(false);
pub(crate) static IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static SENDER: OnceCell<tokio::sync::broadcast::Sender<Command>> = OnceCell::new();
pub(crate) static HANDLE: OnceCell<tokio::task::JoinHandle<()>> = OnceCell::new();

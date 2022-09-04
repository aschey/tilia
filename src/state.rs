use std::sync::atomic::AtomicBool;

use once_cell::sync::OnceCell;

use crate::command::Command;

pub(crate) static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
pub(crate) static IS_CONNECTED: AtomicBool = AtomicBool::new(false);
pub(crate) static SENDER: OnceCell<tokio::sync::mpsc::Sender<Command>> = OnceCell::new();
pub(crate) static HANDLE: OnceCell<tokio::task::JoinHandle<()>> = OnceCell::new();

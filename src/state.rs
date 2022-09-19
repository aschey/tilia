use crate::command::Command;
use once_cell::sync::OnceCell;
use std::sync::{atomic::AtomicU16, RwLock};

pub(crate) struct ConnectState;

impl ConnectState {
    pub(crate) const NOT_CONNECTED: u16 = 1;
    pub(crate) const CONNECTED: u16 = 2;
    pub(crate) const DISABLED: u16 = 3;
}

pub(crate) static IS_INITIALIZED: RwLock<bool> = RwLock::new(false);
pub(crate) static CONNECT_STATE: AtomicU16 = AtomicU16::new(ConnectState::DISABLED);
pub(crate) static SENDER: OnceCell<tokio::sync::mpsc::Sender<Command>> = OnceCell::new();
pub(crate) static HANDLE: OnceCell<tokio::task::JoinHandle<()>> = OnceCell::new();

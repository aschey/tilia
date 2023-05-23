mod writer;
use std::net::SocketAddr;

pub use writer::Writer;
mod worker_guard;
pub use worker_guard::WorkerGuard;
mod filter;
mod server;
mod state;
pub use filter::Filter;
mod client;
pub use client::*;
mod history;

#[derive(Clone)]
pub enum TransportType {
    Ipc(String),
    Tcp(SocketAddr),
}

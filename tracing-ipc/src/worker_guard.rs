use crate::{command::Command, state};

pub struct WorkerGuard;

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        // Don't need to flush if the handle was never created
        if state::HANDLE.get().is_none() {
            return;
        }

        if let Some(sender) = state::SENDER.get() {
            sender.send(Command::Flush).unwrap();
        }
    }
}

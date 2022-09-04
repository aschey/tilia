use std::time::Duration;

use crate::{command::Command, state};

pub struct WorkerGuard;

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        if let Some(sender) = state::SENDER.get() {
            futures::executor::block_on(async {
                sender
                    .send_timeout(Command::Flush, Duration::from_millis(100))
                    .await
                    .unwrap();
            });
        }
    }
}

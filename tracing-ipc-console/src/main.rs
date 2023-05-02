use std::{env::args, error::Error};

use tracing_ipc_console::Console;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = args().collect::<Vec<_>>();
    if let Some(name) = args.get(1) {
        Console::new(name.to_owned()).run().await
    } else {
        Ok(())
    }
}

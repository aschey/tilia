use std::{env::args, error::Error};
use tilia_console::Console;
use tilia_widget::transport::ipc_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = args().collect::<Vec<_>>();
    if let Some(name) = args.get(1) {
        Console::new(ipc_client(name.to_owned())).run().await
    } else {
        Ok(())
    }
}

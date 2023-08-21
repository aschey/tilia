use std::env::args;
use std::error::Error;

use tilia_console::Console;
use tilia_widget::transport::ipc_client;
use tower_rpc::transport::ipc::ConnectionId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = args().collect::<Vec<_>>();
    if let Some(name) = args.get(1) {
        Console::new(ipc_client(ConnectionId(name.to_owned())))
            .run()
            .await
    } else {
        Ok(())
    }
}

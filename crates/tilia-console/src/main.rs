use std::env::args;

use tilia_console::Console;
use tilia_widget::transport::ipc_client;
use tilia_widget::BoxedError;
use tower_rpc::transport::ipc::ServerId;

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    let args = args().collect::<Vec<_>>();
    if let Some(name) = args.get(1) {
        Console::new(ipc_client(ServerId(name.to_owned())))
            .run()
            .await
    } else {
        Err("app name required".into())
    }
}

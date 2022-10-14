use std::env::args;

use tracing_ipc_console::Console;

#[tokio::main]
async fn main() {
    let args = args().collect::<Vec<_>>();
    if let Some(name) = args.get(1) {
        Console::new(name.to_owned()).run().await.unwrap();
    }
}

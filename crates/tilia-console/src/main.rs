use std::{env::args, error::Error};

use tilia_console::Console;
use tower_rpc::{length_delimited_codec, transport::ipc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = args().collect::<Vec<_>>();
    if let Some(name) = args.get(1) {
        let name = name.clone();
        Console::new(move || {
            let name = name.clone();
            Box::pin(async move {
                let client_transport = ipc::connect(name).await?;
                Ok(length_delimited_codec(client_transport))
            })
        })
        .run()
        .await
    } else {
        Ok(())
    }
}

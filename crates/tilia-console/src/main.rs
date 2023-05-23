use std::{env::args, error::Error};

use tilia_console::Console;
use tilia_widget::TransportType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = args().collect::<Vec<_>>();
    if let Some(name) = args.get(1) {
        let transport_type = match name.parse() {
            Ok(addr) => TransportType::Tcp(addr),
            Err(_) => TransportType::Ipc(name.to_owned()),
        };
        Console::new(transport_type).run().await
    } else {
        Ok(())
    }
}

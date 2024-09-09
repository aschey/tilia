use clap::{Parser, ValueEnum};
use tilia_console::Console;
use tilia_widget::transport::docker::{self, docker_client};
use tilia_widget::transport::{ipc_client, tcp_client};
use tilia_widget::BoxedError;
use transport_async::ipc::ServerId;

#[derive(Clone, Debug, ValueEnum)]
pub enum ContainerLogSource {
    Stdout,
    Stderr,
    All,
}

#[derive(Clone, Debug, clap::Parser)]
pub enum Tranport {
    Ipc {
        app_name: String,
    },
    Tcp {
        address: String,
    },
    Container {
        name: String,
        log_source: ContainerLogSource,
    },
}

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    let cli = Tranport::parse();

    match cli {
        Tranport::Ipc { app_name } => {
            Console::new(ipc_client(ServerId::new(app_name)))
                .run()
                .await
        }
        Tranport::Tcp { address } => Console::new(tcp_client(address)).run().await,
        Tranport::Container { name, log_source } => {
            Console::new(docker_client(
                name,
                match log_source {
                    ContainerLogSource::Stdout => docker::LogSource::Stdout,
                    ContainerLogSource::Stderr => docker::LogSource::Stderr,
                    ContainerLogSource::All => docker::LogSource::All,
                },
            ))
            .run()
            .await
        }
    }
}

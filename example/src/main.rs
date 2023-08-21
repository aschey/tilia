use std::env::args;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use tilia::tower_rpc::transport::ipc::{
    self, IpcSecurity, OnConflict, SecurityAttributes, ServerId,
};
use tilia::tower_rpc::transport::CodecTransport;
use tilia::tower_rpc::LengthDelimitedCodec;
use tracing::{debug, error, info, trace, warn, Level};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    if let Some(name) = args().collect::<Vec<_>>().get(1) {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(Level::TRACE.into())
            .add_directive("tokio_util=info".parse().unwrap())
            .add_directive("tokio_tower=info".parse().unwrap());

        let name = name.to_owned();
        let (ipc_writer, mut guard) = tilia::Writer::new(1024, move || {
            let name = name.to_owned();
            Box::pin(async move {
                let transport = ipc::create_endpoint(
                    ServerId(name),
                    SecurityAttributes::allow_everyone_create().unwrap(),
                    OnConflict::Overwrite,
                )
                .unwrap();
                CodecTransport::new(transport, LengthDelimitedCodec)
            })
        });

        tracing_subscriber::registry()
            .with(env_filter)
            .with({
                Layer::new()
                    .compact()
                    .with_writer(ipc_writer)
                    .with_filter(tilia::Filter::default())
            })
            .init();

        let mut rng = thread_rng();
        let levels = [
            Level::TRACE,
            Level::DEBUG,
            Level::INFO,
            Level::WARN,
            Level::ERROR,
        ];
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        while running.load(Ordering::SeqCst) {
            let level = levels.choose(&mut rng).expect("Empty levels").to_owned();
            let sleep_seconds: f64 = rng.gen();
            log(level, (sleep_seconds * 1000.0) as u64).await;
        }
        println!("\nStopping...");
        guard.stop().await.ok();
    }
}

#[tracing::instrument]
async fn log(level: Level, sleep_millis: u64) {
    let err = "oops";

    match level {
        Level::TRACE => trace!("A trace event occurred"),
        Level::DEBUG => debug!("A debug event occurred"),
        Level::INFO => info!("An info event occurred"),
        Level::WARN => warn!("A warn event occurred"),
        Level::ERROR => error!({ error = err }, "An error event occurred:"),
    }

    tokio::time::sleep(Duration::from_millis(sleep_millis)).await;
}

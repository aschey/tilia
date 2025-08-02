use std::env::args;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rand::Rng;
use rand::seq::IndexedRandom;
use tilia::BoxedError;
use tilia::transport_async::Bind;
use tilia::transport_async::codec::{CodecStream, LengthDelimitedCodec};
use tilia::transport_async::ipc::{self, OnConflict, SecurityAttributes, ServerId};
use tracing::{Level, debug, error, info, trace, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    if let Some(name) = args().collect::<Vec<_>>().get(1) {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(Level::TRACE.into())
            .add_directive("tokio_util=info".parse().unwrap())
            .add_directive("tokio_tower=info".parse().unwrap());

        let name = name.to_owned();
        let (ipc_writer, mut guard) = tilia::Writer::new(1024, move || {
            let name = name.to_owned();
            Box::pin(async move {
                let transport = ipc::Endpoint::bind(
                    ipc::EndpointParams::new(
                        ServerId::new(name),
                        SecurityAttributes::allow_everyone_create().unwrap(),
                        OnConflict::Overwrite,
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
                CodecStream::new(transport, LengthDelimitedCodec)
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

        let mut rng = rand::rng();
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
            let sleep_seconds: f64 = rng.random();
            log(level, (sleep_seconds * 1000.0) as u64).await;
        }
        println!("\nStopping...");
        let _ = guard.stop().await;
        Ok(())
    } else {
        Err("app name required".into())
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

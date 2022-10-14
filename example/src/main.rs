use std::{env::args, time::Duration};

use rand::{seq::SliceRandom, thread_rng, Rng};
use tracing::{debug, error, info, trace, warn, Level};
use tracing_subscriber::{fmt::Layer, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    if let Some(name) = args().collect::<Vec<_>>().get(1) {
        let env_filter = EnvFilter::from_default_env().add_directive(Level::TRACE.into());
        let (ipc_writer, _guard) = tracing_ipc::Writer::new(name);
        tracing_subscriber::registry()
            .with(env_filter)
            .with({
                Layer::new()
                    .compact()
                    .with_writer(ipc_writer)
                    .with_filter(tracing_ipc::Filter::default())
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
        loop {
            let level = levels.choose(&mut rng).unwrap().to_owned();
            let sleep_seconds: f64 = rng.gen();
            log(level, (sleep_seconds * 1000.0) as u64).await;
        }
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

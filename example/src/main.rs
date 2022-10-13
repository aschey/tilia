use std::{env::args, time::Duration};

use rand::{seq::SliceRandom, thread_rng, Rng};
use tracing::{debug, error, info, metadata::LevelFilter, trace, warn, Level};
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
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_writer(ipc_writer)
                    .with_filter(tracing_ipc::Filter::new(LevelFilter::TRACE))
            })
            .init();

        let levels = [
            Level::TRACE,
            Level::DEBUG,
            Level::INFO,
            Level::WARN,
            Level::ERROR,
        ];
        let mut rng = thread_rng();

        loop {
            let level = levels.choose(&mut rng).unwrap().to_owned();
            match level {
                Level::TRACE => trace!("A trace event occurred"),
                Level::DEBUG => debug!("A debug event occurred"),
                Level::INFO => info!("An info event occurred"),
                Level::WARN => warn!("A warn event occurred"),
                Level::ERROR => error!("An error event occurred"),
            }
            let rand_num: f64 = rng.gen();

            tokio::time::sleep(Duration::from_millis((rand_num * 1000.0) as u64)).await;
        }
    }
}

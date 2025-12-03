use tracing_subscriber::{EnvFilter, prelude::*};

pub fn init_tracing_subscriber(filter: EnvFilter) {
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .init();
}

use tracing_subscriber::{EnvFilter, prelude::*};
use tracing_web::MakeWebConsoleWriter;

pub fn init_tracing_subscriber(filter: EnvFilter) {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .init();
}

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub struct Logger;

impl Logger {
    pub fn init() {
        let fmt_layer = fmt::layer()
            .json()
            // .with_span_events(fmt::format::FmtSpan::FULL) // to verbose, for now
            .with_level(true)
            .with_line_number(true);

        let filter_layer = EnvFilter::try_from_default_env()
            // .unwrap_or_else(|_| EnvFilter::new("cloudflare_ddns=debug,axum=info,tower_http=info"));
            .unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    }
}

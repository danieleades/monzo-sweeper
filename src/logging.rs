use tracing_subscriber::filter::EnvFilter;

use crate::config;

pub fn set_up(verbosity: u8) {
    let formatter = tracing_subscriber::fmt::format::debug_fn(|writer, _field, value| {
        write!(writer, "{:?}", value)
    });

    let filter = EnvFilter::try_new("warn").unwrap().add_directive(
        format!("{}={}", config::BIN_NAME, max_level(verbosity))
            .parse()
            .unwrap(),
    );

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .with_target(true)
        .fmt_fields(formatter)
        .init();
}

fn max_level(verbosity: u8) -> &'static str {
    match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    }
}

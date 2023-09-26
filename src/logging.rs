// Don't uses logging if `RUST_LOG` is unset
pub fn init_logging() {
    let rust_log = std::env::var("RUST_LOG");
    if matches!(rust_log.as_deref(), Ok("trace")) {
        tracing_subscriber::fmt::init();
    }
}
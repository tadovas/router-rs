use std::env;
use std::env::VarError;

// setups default env logger level to info if env var is not present
pub fn init_with_default_level() {
    if let Err(VarError::NotPresent) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init()
}

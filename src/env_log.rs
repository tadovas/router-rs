use std::env;
use std::env::VarError;

pub fn setup_env_logger() {
    if let Err(VarError::NotPresent) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init()
}

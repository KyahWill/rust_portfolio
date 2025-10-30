use std::env;
use crate::server::Server;
pub mod server;
pub mod config;
pub mod static_files;

fn main() {

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let config = match config::load_config(&config_path) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Failed to load config: {:?}", err);
            std::process::exit(1);
        }
    };

    let mut server:Server = Server {
        port: config.server.port.to_string(),
        host: config.server.host.clone(),
        address: None,
        listener: None,
        config: Some(config),
        resolver: None,
    };

   server.setup_server();
}
use std::env;
use dotenv::dotenv;

use crate::server::Server;
pub mod server;

fn main() {
    dotenv().expect(".env file not found");

    let mut server:Server = Server {
        port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
        host: env::var("HOST").unwrap_or_else(|_|"0.0.0.0".to_string()),
        address: None,
        listener: None,
    };

   server.setup_server();
}
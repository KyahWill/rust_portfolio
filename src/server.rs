use std::io::{BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::AppConfig;
use crate::static_files::{resolve_content_type, StaticFileResolver};

pub struct Route {
    
}
pub struct Server {
    pub host: String,
    pub port: String,
    pub address: Option<String>,
    pub listener: Option<TcpListener>,
    pub config: Option<AppConfig>,
    pub resolver: Option<StaticFileResolver>,
}

impl Server {
    pub fn setup_server(&mut self) {
        self.address = Some(format!("{}:{}", self.host, self.port));
        // initialize resolver if config present
        if let Some(cfg) = &self.config {
            match StaticFileResolver::from_config(&cfg.static_cfg) {
                Ok(res) => {
                    self.resolver = Some(res);
                }
                Err(err) => {
                    eprintln!("ERROR initializing resolver: {:?}", err);
                }
            }
        }
        self.setup_listener();
    }

    fn setup_listener(&mut self) {
        match &self.address {
            Some(address) => {
                let listener_result = TcpListener::bind(address);
                match listener_result {
                    Ok(listener) => {
                        println!("Listening to: {}", address);
                        self.listener = Some(listener);
                        self.listen();
                    }
                    Err(error) => {
                        eprintln!("ERROR: {:?}", error);
                    }
                }
            }
            None => {}
        }
    }

    fn listen(&self) {
        match &self.listener {
            Some(listener) => {
                for stream in listener.incoming() {
                    match stream {
                        Ok(tcp_stream) => {
                            self.handle_stream(tcp_stream);
                        }
                        Err(error) => {
                            eprintln!("ERRLR: {:?}", error);
                        }
                    }
                }
            }
            None => {}
        }
    }

    fn handle_stream(&self, mut tcp_stream: TcpStream) {
        let buf_reader = BufReader::new(&tcp_stream);
        let http_request: Vec<String> = buf_reader
            .lines()
            .map(|result| match result {
                Ok(result) => return result,
                Err(error) => {
                    eprintln!("ERROR: {:?}", error);
                    panic!();
                }
            })
            .take_while(|line| !line.is_empty())
            .collect();

        let http_header: Vec<&str>= http_request[0].split(' ').collect();
        let route = http_header[1];
        println!("ROUTE {}",route);

        // Default response
        let mut response: Vec<u8> = b"HTTP/1.1 404 ERROR\r\nContent-Length: 13\r\n\r\nFile Not Found".to_vec();

        if let Some(cfg) = &self.config {
            if let Some(resolver) = &self.resolver {
                match resolver.resolve(route) {
                    Ok(path) => {
                        match fs::read(&path) {
                            Ok(bytes) => {
                                let content_type = resolve_content_type(&path, &cfg.content_types);
                                let headers = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
                                    bytes.len(), content_type
                                );
                                let mut buf = headers.into_bytes();
                                buf.extend_from_slice(&bytes);
                                response = buf;
                            }
                            Err(_e) => {
                                response = b"HTTP/1.1 404 ERROR\r\nContent-Length: 13\r\n\r\nFile Not Found".to_vec();
                            }
                        }
                    }
                    Err(_e) => {
                        response = b"HTTP/1.1 404 ERROR\r\nContent-Length: 13\r\n\r\nFile Not Found".to_vec();
                    }
                }
            }
        }
        
        match tcp_stream.write_all(&response) {
            Ok(_result) => {}
            Err(error) => {
                eprintln!("ERROR: {:?}", error)
            }
        };
    }
}


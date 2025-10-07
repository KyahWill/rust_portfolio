use std::fs;
use std::io::{BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;

pub struct Route {
    
}
pub struct Server {
    pub host: String,
    pub port: String,
    pub address: Option<String>,
    pub listener: Option<TcpListener>,
}

impl Server {
    pub fn setup_server(&mut self) {
        self.address = Some(format!("{}:{}", self.host, self.port));
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

        let mut status_line: &str = "HTTP/1.1 200 OK";
        let mut contents: String = String::new();
        let mut length = contents.len();
        let mut response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        match route {
            "/" => {
                let file_string_result = fs::read_to_string("public/index.html");
                match file_string_result {
                    Ok(string) => {
                        contents = string;
                        status_line = "HTTP/1.1 200 OK";
                length = contents.len();
                        response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
                    }
                    Err(_error) => {
                        contents = "File Not Found".to_string();
                        status_line = "HTTP/1.1 404 ERROR";
                length = contents.len();
                        response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
                    }
                }
            },
            "/index.css"=> {
                let file_string_result = fs::read_to_string("public/index.css");
                match file_string_result {
                    Ok(string) => {
                        contents = string;
                        status_line = "HTTP/1.1 200 OK";
                        length = contents.len();
                        response = format!("{status_line}\r\nContent-Length: {length} \r\nContent-Type: text/css\r\n\r\n{contents}");
                        println!("Response {}",response);
                    }
                    Err(_error) => {
                        contents = "File Not Found".to_string();
                        status_line = "HTTP/1.1 404 ERROR";
                length = contents.len();
                        response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
                    }
                }

            },
            _ => {

                contents = "File Not Found".to_string();
                status_line = "HTTP/1.1 404 ERROR";
                length = contents.len();
                response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
            }
        }
        
        match tcp_stream.write_all(response.as_bytes()) {
            Ok(_result) => {}
            Err(error) => {
                eprintln!("ERROR: {:?}", error)
            }
        };
    }
}


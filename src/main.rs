use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};

fn handle_stream(tcp_stream: TcpStream) {
    let buf_reader = BufReader::new(&tcp_stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
}

fn listen(listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                handle_stream(tcp_stream);
            }
            Err(error) => {
                eprintln!("ERRLR: {:?}", error);
            }
        }
    }
}
fn main() {
    let address = "0.0.0.0:8000";
    let listener_result = TcpListener::bind(address);
    match listener_result {
        Ok(listener) => {
            println!("Listening to: {}", address);
            listen(listener);
        }
        Err(error) => {
            eprintln!("ERROR: {:?}", error);
        }
    }
}

pub mod lib;

use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use lib::ThreadPool;
use http::{Request, Response, StatusCode};
use quick_js::{Context, JsValue};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(64);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|result| {
            handle_connection(stream, result);
        });
    }
}

fn handle_connection(mut stream: TcpStream, result: JsValue) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    stream.write(b"HTTP/1.1 200 OK\r\n").unwrap();
    stream.write(b"Content-Type: text/plain\r\n").unwrap();
    stream.write(b"Content-Length: 6\r\n\r\n").unwrap();
    stream.write(format!("{:?}", result.as_str().unwrap()).as_bytes()).unwrap();
}

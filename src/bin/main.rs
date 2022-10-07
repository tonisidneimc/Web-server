extern crate web_server;
use web_server::ThreadPool;

use std::{
    thread,
    time::Duration,
    io::prelude::*,
    net::{TcpStream, TcpListener},
    fs::File
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); // localhost and port 7878
    let pool = ThreadPool::new(5); // create a configurable thread pool with 5 threads

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    //println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n"; // for debugging purpose only

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) { 
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

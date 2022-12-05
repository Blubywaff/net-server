use net_server::threading::ThreadPool;
use std::{
    fs,
    io::{prelude::*, BufRead, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7001") {
        Ok(l) => l,
        Err(e) => panic!("{e}"),
    };
    let pool = ThreadPool::new(4).expect("Could not create threads");
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                println!("{e}");
                continue;
            },
        };
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    stream.write_all(b"hello! You wrote:\r\n").unwrap();
    stream.write_all(request_line.as_bytes()).unwrap();
}

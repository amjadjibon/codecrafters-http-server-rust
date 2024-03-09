use std::io::{Result, Write};
use std::net::{TcpListener, TcpStream };

fn handle_client(mut stream: TcpStream) -> Result<()> {
    stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");

                handle_client(_stream).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

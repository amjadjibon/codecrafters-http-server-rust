use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream };

struct Request {
    method: String,
    path: String,
    protocol: String,
}

impl Request {
    fn new(method: String, path: String, protocol: String) -> Self {
        Self {
            method,
            path,
            protocol,
        }
    }

    fn from_buffer(buffer: &[u8]) -> Option<Self> {
        let request = String::from_utf8_lossy(buffer);
        let mut parts = request.lines();
        let request_line = parts.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?;
        let path = parts.next()?;
        let protocol = parts.next()?;
        Some(Self::new(
            method.to_string(),
            path.to_string(),
            protocol.to_string()),
        )
    }
}

fn respond_with_status_ok(stream: &mut TcpStream) -> Result<()> {
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes())
}

fn respond_with_not_found(stream: &mut TcpStream) -> Result<()> {
    let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
    stream.write_all(response.as_bytes())
}

fn handle_protocol_error(stream: &mut TcpStream) -> Result<()> {
    let response = "HTTP/1.1 505 HTTP VERSION NOT SUPPORTED\r\n\r\n";
    stream.write_all(response.as_bytes())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request = Request::from_buffer(&buffer).unwrap();

    if request.protocol != "HTTP/1.1" {
        return handle_protocol_error(&mut stream);
    }

    if request.method == "GET" && request.path == "/"{
        return respond_with_status_ok(&mut stream);
    } else {
        return respond_with_not_found(&mut stream);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_from_buffer() {
        let buffer = b"GET / HTTP/1.1\r\n";
        let request = Request::from_buffer(&buffer[..]).unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/");
        assert_eq!(request.protocol, "HTTP/1.1");
    }
}

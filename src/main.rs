use std::collections::HashMap;
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream };

struct Request {
    method: String,
    path: String,
    protocol: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    fn new(method: String,
        path: String,
        protocol: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            protocol,
            headers,
            body,
        }
    }

    fn from_buffer(buffer: &[u8]) -> Option<Self> {
        let request = String::from_utf8_lossy(buffer);
        let mut lines = request.lines();
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?;
        let path = parts.next()?;
        let protocol = parts.next()?;

        // parse headers and body
        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            let mut header_parts = line.splitn(2, ':');
            let key = header_parts.next()?.trim();
            let value = header_parts.next()?.trim();
            headers.insert(key.to_string(), value.to_string());
        }

        let mut body = String::new();
        if let Some(content_length) = headers.get("Content-Length") {
            let content_length = content_length.parse::<usize>().ok()?;
            let body_start = request.find("\r\n\r\n")? + 4;
            let body_end = body_start + content_length;
            body = request[body_start..body_end].to_string();
        }

        Some(Self::new(
            method.to_string(),
            path.to_string(),
            protocol.to_string(),
            headers,
            body,
        ))
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

fn respond_echo(stream: &mut TcpStream, request: &Request) -> Result<()> {
    let message = request.path.trim_start_matches("/echo/").trim();
    let content_type = "text/plain";
    let content_length = message.len();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        content_type, content_length, message
    );
    stream.write_all(response.as_bytes())
}

fn respond_user_agent(stream: &mut TcpStream, request: &Request) -> Result<()> {
    let user_agent = match request.headers.get("User-Agent") {
        Some(user_agent) => user_agent,
        None => "Unknown",
    };


    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        "text/plain",
        user_agent.len(),
        user_agent,
    );

    stream.write_all(response.as_bytes())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    // let request = Request::from_buffer(&buffer).unwrap();
    let request = match Request::from_buffer(&buffer) {
        Some(request) => request,
        None => return handle_protocol_error(&mut stream),
    };

    match request.protocol.as_str() {
        "HTTP/1.1" => (),
        _ => return handle_protocol_error(&mut stream),
    }

    // log request to console
    println!("Method:{}\nPath: {}\nProtocol: {}", request.method, request.path, request.protocol);
    println!("Headers:");
    for (key, value) in &request.headers {
        println!("{}: {}", key, value);
    }
    println!("Body\n{}", request.body);

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => respond_with_status_ok(&mut stream),
        ("GET", path) if path.starts_with("/echo/") => respond_echo(&mut stream, &request),
        ("GET", "/user-agent") => respond_user_agent(&mut stream, &request),
        _ => respond_with_not_found(&mut stream),
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
        let buffer = b"GET / HTTP/1.1\r\nContent-Length: 5\r\n\r\nHello";
        let request = Request::from_buffer(&buffer[..]).unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/");
        assert_eq!(request.protocol, "HTTP/1.1");
        assert_eq!(request.headers.get("Content-Length"), Some(&"5".to_string()));
        assert_eq!(request.body, "Hello");
    }
}

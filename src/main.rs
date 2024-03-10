mod flags;
mod request;

use std::io::Result;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use flags::parse_flags;
use request::Request;


async fn respond_with_status_ok(stream: &mut TcpStream) -> Result<()> {
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).await
}

async fn respond_with_not_found(stream: &mut TcpStream) -> Result<()> {
    let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
    stream.write_all(response.as_bytes()).await
}

async fn handle_protocol_error(stream: &mut TcpStream) -> Result<()> {
    let response = "HTTP/1.1 505 HTTP VERSION NOT SUPPORTED\r\n\r\n";
    stream.write_all(response.as_bytes()).await
}

async fn respond_echo(stream: &mut TcpStream, request: &Request) -> Result<()> {
    let message = request.path.trim_start_matches("/echo/").trim();
    let content_type = "text/plain";
    let content_length = message.len();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        content_type, content_length, message
    );
    stream.write_all(response.as_bytes()).await
}

async fn respond_user_agent(stream: &mut TcpStream, request: &Request) -> Result<()> {
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

    stream.write_all(response.as_bytes()).await
}

async fn respond_file(stream: &mut TcpStream, request: &Request, root_path: String) -> Result<()> {
    // remove /files prefix
    let request_path = match request.path.strip_prefix("/files") {
        Some(path) => path,
        None => return respond_with_not_found(stream).await,
        
    };

    let file_path = format!("{}{}", root_path, request_path);
    if !Path::new(&file_path).exists() {
        return respond_with_not_found(stream).await;
    }
    
    let file_content = match tokio::fs::read(&file_path).await {
        Ok(content) => content,
        Err(_) => return respond_with_not_found(stream).await,
    };

    let content_type = "application/octet-stream";
    let content_length = file_content.len();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        content_type, content_length
    );

    stream.write_all(response.as_bytes()).await?;
    stream.write_all(&file_content).await
}

async fn handle_client(mut stream: TcpStream, file_path: String) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await?;

    let request = match Request::from_buffer(&buffer) {
        Some(request) => request,
        None => return handle_protocol_error(&mut stream).await,
    };

    match request.protocol.as_str() {
        "HTTP/1.1" => (),
        _ => return handle_protocol_error(&mut stream).await,
    }

    // log request to console
    println!("Method:{}\nPath: {}\nProtocol: {}", request.method, request.path, request.protocol);
    println!("Headers:");
    for (key, value) in &request.headers {
        println!("{}: {}", key, value);
    }
    println!("Body\n{}", request.body);

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => respond_with_status_ok(&mut stream).await,
        ("GET", path) if path.starts_with("/echo/") => respond_echo(&mut stream, &request).await,
        ("GET", "/user-agent") => respond_user_agent(&mut stream, &request).await,
        ("GET", path) if path.starts_with("/files/") => respond_file(&mut stream, &request, file_path).await,
        _ => respond_with_not_found(&mut stream).await,
    }
}

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let args = std::env::args().collect::<Vec<String>>();
    let flags = parse_flags(&args);
    let directory = match flags.get("directory") {
        Some(directory) => directory.to_string(),
        None => "public".to_string(),
    };

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("accepted new connection");
                let directory = directory.clone();
                tokio::spawn(async move {
                    handle_client(stream, directory).await.unwrap();
                });
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

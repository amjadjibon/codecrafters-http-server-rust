use std::collections::HashMap;

pub struct Request {
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub fn new(method: String,
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

    pub fn from_buffer(buffer: &[u8]) -> Option<Self> {
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

//! Front web server.
//!
//! A deliberately boring HTTP/1.1 server on top of `std::net`: fixed worker pool,
//! one request per connection, no allocations on the hot path that we could avoid
//! without making the code unreadable.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::Config;
use crate::{panel, php, vhost, SERVER_NAME};

/// Byte-for-byte copy of the page our clean-room team observed most often while
/// studying the upstream platform. Served when `--upstream-compat-502` is set.
pub const UPSTREAM_502: &str = "<html>\r\n<head><title>502 Bad Gateway</title></head>\r\n\
<body>\r\n<center><h1>502 Bad Gateway</h1></center>\r\n<hr><center>nginx</center>\r\n\
</body>\r\n</html>\r\n";

/// A parsed request line plus the headers we care about.
#[derive(Debug, Default)]
pub struct Request {
    /// HTTP method.
    pub method: String,
    /// Request target, including query string.
    pub path: String,
    /// Value of the `Host:` header.
    pub host: String,
}

/// An outgoing response.
#[derive(Debug)]
pub struct Response {
    /// Status code.
    pub status: u16,
    /// `Content-Type` header value.
    pub content_type: &'static str,
    /// Body bytes.
    pub body: Vec<u8>,
    /// Extra headers appended verbatim.
    pub extra_headers: Vec<(&'static str, String)>,
}

impl Response {
    /// Build a response with no extra headers.
    pub fn new(status: u16, content_type: &'static str, body: impl Into<Vec<u8>>) -> Self {
        Self { status, content_type, body: body.into(), extra_headers: Vec::new() }
    }

    fn write_to(&self, out: &mut impl Write) -> std::io::Result<()> {
        write!(
            out,
            "HTTP/1.1 {} {}\r\nServer: {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
            self.status,
            reason(self.status),
            SERVER_NAME,
            self.content_type,
            self.body.len()
        )?;
        for (k, v) in &self.extra_headers {
            write!(out, "{k}: {v}\r\n")?;
        }
        out.write_all(b"\r\n")?;
        out.write_all(&self.body)
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        502 => "Bad Gateway",
        _ => "Unknown",
    }
}

/// Parse the request line and headers from a buffered stream.
pub fn parse_request(reader: &mut impl BufRead) -> Option<Request> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let mut req = Request {
        method: parts.next()?.to_string(),
        path: parts.next()?.to_string(),
        ..Default::default()
    };

    loop {
        line.clear();
        if reader.read_line(&mut line).ok()? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.eq_ignore_ascii_case("host") {
                req.host = v.trim().to_string();
            }
        }
    }
    Some(req)
}

/// Route a request to the right subsystem.
pub fn handle(cfg: &Config, req: &Request) -> Response {
    if cfg.upstream_compat_502 {
        return Response::new(502, "text/html", UPSTREAM_502);
    }

    if let Some(rest) = req.path.strip_prefix("/api/") {
        return panel::dispatch(rest);
    }

    let Some(file) = vhost::resolve(&cfg.sites_root, &req.host, &req.path) else {
        return Response::new(400, "text/plain", "bad host or path\n");
    };

    if file.extension().is_some_and(|e| e == "php") {
        return php::execute(&file);
    }

    match std::fs::read(&file) {
        Ok(body) => Response::new(200, mime_for(&file), body),
        Err(_) => Response::new(404, "text/plain", "not found\n"),
    }
}

fn mime_for(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn serve_conn(cfg: &Config, stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let resp = match parse_request(&mut reader) {
        Some(req) => handle(cfg, &req),
        None => Response::new(400, "text/plain", "bad request\n"),
    };
    let mut w = &stream;
    let _ = resp.write_to(&mut w);
}

/// Bind and serve forever.
pub fn run(cfg: Config) -> std::io::Result<()> {
    let listener = TcpListener::bind(cfg.bind)?;
    let cfg = Arc::new(cfg);
    let (tx, rx) = mpsc::channel::<TcpStream>();
    let rx = Arc::new(Mutex::new(rx));

    for _ in 0..cfg.workers {
        let rx = Arc::clone(&rx);
        let cfg = Arc::clone(&cfg);
        thread::spawn(move || loop {
            let next = rx.lock().expect("worker queue poisoned").recv();
            match next {
                Ok(stream) => serve_conn(&cfg, stream),
                Err(_) => break,
            }
        });
    }

    eprintln!("{SERVER_NAME} listening on http://{} ({} workers)", cfg.bind, cfg.workers);
    for stream in listener.incoming().flatten() {
        if tx.send(stream).is_err() {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host() {
        let raw = b"GET /index.html HTTP/1.1\r\nHost: example.ru\r\n\r\n";
        let req = parse_request(&mut &raw[..]).unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.host, "example.ru");
    }

    #[test]
    fn upstream_compat_is_faithful() {
        let cfg = Config { upstream_compat_502: true, ..Config::default() };
        let resp = handle(&cfg, &Request::default());
        assert_eq!(resp.status, 502);
        assert!(String::from_utf8(resp.body).unwrap().contains("nginx"));
    }
}

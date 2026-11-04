//! A tiny local HTTP server that records the requests it receives and answers
//! each with the same reply. The integration tests that check what the client
//! sends use it instead of ESI.

#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// One request as the server received it.
#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    /// The path and the query string, as sent.
    pub target: String,
    headers: Vec<(String, String)>,
    pub body: String,
}

impl Recorded {
    fn parse(raw: &str) -> Self {
        let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
        let mut lines = head.lines();
        let mut request_line = lines.next().unwrap_or_default().split_whitespace();
        let method = request_line.next().unwrap_or_default().to_owned();
        let target = request_line.next().unwrap_or_default().to_owned();
        let headers = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.trim().to_lowercase(), value.trim().to_owned()))
            .collect();
        Self {
            method,
            target,
            headers,
            body: body.to_owned(),
        }
    }

    /// The value of a header (the name in lowercase).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or_default()
    }

    /// The `key=value` pairs of the query string, sorted, as sent.
    pub fn query(&self) -> Vec<String> {
        Self::sorted_pairs(self.target.split_once('?').map_or("", |(_, query)| query))
    }

    /// The `key=value` pairs of a form body, sorted, as sent.
    pub fn form(&self) -> Vec<String> {
        Self::sorted_pairs(&self.body)
    }

    /// The body as JSON.
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).expect("the body is not JSON")
    }

    fn sorted_pairs(text: &str) -> Vec<String> {
        let mut pairs: Vec<String> = text
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(str::to_owned)
            .collect();
        pairs.sort();
        pairs
    }
}

/// The answer the server gives to every request.
#[derive(Clone)]
pub struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Reply {
    /// A `200` with a JSON body.
    pub fn json(body: &str) -> Self {
        Self {
            status: 200,
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: body.to_owned(),
        }
    }

    /// No answer at all: the server reads the request and keeps the connection
    /// open without writing, to test timeouts.
    pub fn never() -> Self {
        Self {
            status: 0,
            headers: Vec::new(),
            body: String::new(),
        }
    }

    /// An empty answer with the given status.
    pub fn status(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: String::new(),
        }
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// The raw answer; `{base}` in the body stands for the address of the server.
    fn render(&self, base: &str) -> String {
        if self.status == 0 {
            return String::new();
        }
        let body = self.body.replace("{base}", base);
        let mut text = format!("HTTP/1.1 {} Test\r\n", self.status);
        for (name, value) in &self.headers {
            text.push_str(&format!("{name}: {value}\r\n"));
        }
        text.push_str(&format!(
            "Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        ));
        text
    }
}

pub struct Server {
    /// The address to give the client, ending in `/`.
    pub url: String,
    requests: Arc<Mutex<Vec<Recorded>>>,
}

impl Server {
    /// A server that gives every request the same reply.
    pub async fn start(reply: Reply) -> Self {
        Self::routes(vec![("/", reply)]).await
    }

    /// A server with a reply per path prefix; the first prefix that matches
    /// wins and a path that matches none gets a `404`.
    pub async fn routes(routes: Vec<(&str, Reply)>) -> Self {
        Self::launch(
            routes
                .into_iter()
                .map(|(prefix, reply)| (prefix, vec![reply]))
                .collect(),
        )
        .await
    }

    /// A server that answers the first request with the first reply, the
    /// second with the second, and so on; the last reply repeats.
    pub async fn sequence(replies: Vec<Reply>) -> Self {
        Self::launch(vec![("/", replies)]).await
    }

    async fn launch(routes: Vec<(&str, Vec<Reply>)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let requests: Arc<Mutex<Vec<Recorded>>> = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        let answers: Vec<(String, Vec<String>)> = routes
            .into_iter()
            .map(|(prefix, replies)| {
                let texts = replies.iter().map(|reply| reply.render(&url)).collect();
                (prefix.to_owned(), texts)
            })
            .collect();
        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let seen = Arc::clone(&seen);
                let answers = answers.clone();
                tokio::spawn(async move {
                    let raw = Self::read_request(&mut socket).await;
                    let recorded = Recorded::parse(&raw);
                    let answer = {
                        let mut recorded_so_far = seen.lock().unwrap();
                        let route = answers
                            .iter()
                            .find(|(prefix, _)| recorded.path().starts_with(prefix.as_str()));
                        let answer = route.map_or_else(
                            || Reply::status(404).render(""),
                            |(prefix, texts)| {
                                let before = recorded_so_far
                                    .iter()
                                    .filter(|request| request.path().starts_with(prefix.as_str()))
                                    .count();
                                texts[before.min(texts.len() - 1)].clone()
                            },
                        );
                        recorded_so_far.push(recorded);
                        answer
                    };
                    if answer.is_empty() {
                        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    }
                    let _ = socket.write_all(answer.as_bytes()).await;
                });
            }
        });
        Self { url, requests }
    }

    /// Reads the head and as much body as `Content-Length` announces.
    async fn read_request(socket: &mut tokio::net::TcpStream) -> String {
        let mut data: Vec<u8> = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let read = socket.read(&mut buffer).await.unwrap_or(0);
            if read == 0 {
                break;
            }
            data.extend_from_slice(&buffer[..read]);
            let Some(head_end) = data.windows(4).position(|window| window == b"\r\n\r\n") else {
                continue;
            };
            let head = String::from_utf8_lossy(&data[..head_end]).to_lowercase();
            let length = head
                .lines()
                .find_map(|line| line.strip_prefix("content-length:"))
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if data.len() >= head_end + 4 + length {
                break;
            }
        }
        String::from_utf8_lossy(&data).into_owned()
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.requests.lock().unwrap().clone()
    }

    /// The only request received; fails if there were none or several.
    pub fn only(&self) -> Recorded {
        let requests = self.requests();
        assert_eq!(requests.len(), 1, "expected one request: {requests:#?}");
        requests[0].clone()
    }
}

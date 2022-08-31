use std::{collections::HashMap};
use anyhow::{Result, bail, anyhow};

pub enum HttpRequestMethod {
    GET,
    POST,
    DELETE,
}

pub struct HttpRequest {
    method: HttpRequestMethod,
    uri: String,
    version: String,
    headers: HashMap<String, String>,
    content: Vec<u8>
}

pub struct HttpResponse {
    version: String,
    status: u32,
    reason: String,
    headers: Vec<(String, String)>,
    content: Vec<u8>
}

pub struct HostnameHandler {
    hosts_to_hashes: HashMap<String, String>,
}

impl TryFrom<String> for HttpRequest {
    type Error = anyhow::Error;

    fn try_from(request: String) -> Result<HttpRequest> {
        parse_http_request(&request)
    }
}

fn parse_http_request(request: &String) -> Result<HttpRequest> {
    let lines: Vec<&str> = request.lines().collect();
    // strip crlf off end of line so it doesn't get parsed as part of http ver
    let start_line: Vec<&str> = lines[0].split_whitespace().collect();
    if start_line.len() != 3 {
        bail!("Recieved badly formed HTTP request")
    }

    let req_type = match start_line[0] {
        "GET" => HttpRequestMethod::GET,
        "POST" => HttpRequestMethod::POST,
        "DELETE" => HttpRequestMethod::DELETE,
        _ => bail!("Encountered unknown HTTP request method")
    };

    let mut headers = HashMap::new();

    let mut idx = 1;
    while lines[idx] != "\r\n" {
        let mut key_value = lines[idx].split(":");
        let key = key_value.next().ok_or(anyhow!("Bad header line"))?;
        let value = key_value.next().ok_or(anyhow!("Bad header line"))?;
        headers.insert(key.to_string(), value.to_string());

        idx += 1;
    }

    // now process the body
    let mut content = Vec::new();
    // hopefully this takes just the remaining lines
    for line in lines {
        content.extend(line.bytes());
    }

    Ok(HttpRequest { method: req_type, uri: start_line[1].to_string(), version: start_line[2].to_string(), headers, content })
}


impl HostnameHandler {
    pub fn new() -> Self {
        Self { hosts_to_hashes: HashMap::new() }
    }

    pub fn handle_request(&mut self, parsed_request: &HttpRequest) -> HttpResponse {
        // need to parse out the hostname in the request
        // uh, query parameters?
        match parsed_request.method {
            HttpRequestMethod::GET => {

            }
            HttpRequestMethod::POST => {

            }
            HttpRequestMethod::DELETE => {
                HostnameHandler::handle_delete(parsed)
            }
        }
    }

    fn handle_get(hostname: String) -> Result<()> {

        todo!()
    }

    fn handle_post(hostname: String, host_value: String) -> Result<()> {

        todo!()
    }

    fn handle_delete(hostname: String) {
        // there seem to be security issues here
        // need to store more data than the current model allows to verify
        // that it is indeed the original host requesting to delete their entry
        // and not someone else
        unimplemented!()
    }
}

impl Into<Vec<u8>> for HttpResponse {
    fn into(self) -> Vec<u8> {
        let mut out = String::new();

        out.push_str(&format!("{} {} {}\r\n", self.version, self.status, self.reason));
        for (key, val) in self.headers {
            out.push_str(&format!("{}: {}", key, val));
        }

        let mut out: Vec<u8> = out.into_bytes();
        out.extend(self.content);

        out
    }
}
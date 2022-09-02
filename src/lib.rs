use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug)]
pub enum HttpRequestMethod {
    GET,
    POST,
    DELETE,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpRequestMethod,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub content: Vec<u8>,
}

#[derive(Default)]
pub struct HttpResponse {
    pub version: String,
    pub status: u32,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub content: Vec<u8>,
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

impl HttpResponse {
    pub fn bad_request() -> Self {
        HttpResponse {
            version: "HTTP/1.1".into(),
            status: 400,
            reason: "Bad Request".into(),
            ..Default::default()
        }
    }

    pub fn not_found() -> Self {
        HttpResponse {
            version: "HTTP/1.1".into(),
            status: 404,
            reason: "Not Found".into(),
            ..Default::default()
        }
    }
}

fn parse_http_request(request: &str) -> Result<HttpRequest> {
    let lines: Vec<&str> = request.split("\r\n").collect(); // only split crlf, not \n
    if lines.is_empty() {
        bail!("Unable to parse HTTP Requst: Empty")
    }
    // strip crlf off end of line so it doesn't get parsed as part of http ver
    let start_line: Vec<&str> = lines[0].split_whitespace().collect();
    if start_line.len() != 3 {
        bail!("Recieved badly formed HTTP request")
    }

    let req_type = match start_line[0] {
        "GET" => HttpRequestMethod::GET,
        "POST" => HttpRequestMethod::POST,
        "DELETE" => HttpRequestMethod::DELETE,
        _ => bail!("Encountered unknown HTTP request method"),
    };

    let mut headers = HashMap::new();

    let mut idx = 1;
    while idx < lines.len() && !lines[idx].starts_with('\n') {
        let mut key_value = lines[idx].split(':');
        let key = key_value.next().ok_or_else(|| anyhow!("Bad header line"))?;
        let value = key_value
            .next()
            .ok_or_else(|| anyhow!("Bad header line"))?
            .trim(); // leading and trailing whitespace is optional for header fields
        headers.insert(key.to_string().to_lowercase(), value.to_string());

        idx += 1;
    }

    // now process the body
    let mut content = Vec::new();

    for line in lines[idx..].iter() {
        content.extend(line.bytes());
    }

    Ok(HttpRequest {
        method: req_type,
        uri: start_line[1].to_string(),
        version: start_line[2].to_string(),
        headers,
        content,
    })
}

/// Parse query params from a uri
fn parse_query_params(uri: &str) -> HashMap<String, String> {
    let params_block = uri.rsplit_once('?');
    let params_block = match params_block {
        Some((_, s2)) => s2,
        None => return HashMap::new(),
    };
    parse_key_value_list(params_block)
}

/// Parse form parameters from a POST request
/// # Preconditions
/// - `post_request` is an `HttpRequest` with method `HttpRequestMethod::POST`
/// - the post request has been made with the header
fn parse_form_params(post_request: &HttpRequest) -> HashMap<String, String> {
    let out = HashMap::new();
    if let HttpRequestMethod::POST = post_request.method {
        if let Some(t) = post_request.headers.get("content-type") {
            if t != "application/x-www-form-urlencoded" {
                return out;
            }
        } else {
            return out;
        }
        // the body should be in key=value list format now
        parse_key_value_list(&String::from_utf8_lossy(&post_request.content))
    } else {
        out
    }
}

fn parse_key_value_list(list: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let param_pairs = list.split('&');
    // now pairs of the form `key=value`
    for pair in param_pairs {
        let equals_index = match pair.find('=') {
            Some(i) => i,
            None => continue,
        };
        out.insert(
            pair[..equals_index].to_string(),
            pair[equals_index + 1..].to_string(),
        );
    }
    out
}

impl HostnameHandler {
    pub fn new() -> Self {
        Self {
            hosts_to_hashes: HashMap::new(),
        }
    }

    pub fn handle_request(&mut self, parsed_request: &HttpRequest) -> HttpResponse {
        println!("Received request:\n{parsed_request:?}");
        println!("{}", String::from_utf8_lossy(&parsed_request.content));
        // need to parse out the hostname in the request
        // done either via query parameters or body parameters
        let query_params = parse_query_params(&parsed_request.uri);
        let form_params = if let HttpRequestMethod::POST = parsed_request.method {
            parse_form_params(parsed_request)
        } else {
            HashMap::new()
        };

        let result = match parsed_request.method {
            HttpRequestMethod::GET => {
                let hostname = match query_params.get("hostname") {
                    Some(name) => name,
                    None => return HttpResponse::bad_request(),
                };
                self.handle_get(hostname)
            }
            HttpRequestMethod::POST => {
                let hostname = match form_params.get("hostname") {
                    Some(name) => name,
                    None => return HttpResponse::bad_request(),
                };
                let host_value = match form_params.get("host_value") {
                    Some(value) => value,
                    None => return HttpResponse::bad_request(),
                };
                self.handle_post(hostname.clone(), host_value.clone())
            }
            HttpRequestMethod::DELETE => {
                let hostname = match query_params.get("hostname") {
                    Some(name) => name,
                    None => return HttpResponse::bad_request(),
                };
                self.handle_delete(hostname)
            }
        };

        result
    }

    fn handle_get(&self, hostname: &String) -> HttpResponse {
        let value = self.hosts_to_hashes.get(hostname);
        match value {
            Some(value) => {
                let headers: Vec<(String, String)> = vec![
                    ("content-type".into(), "text/plain".into()),
                    ("content-length".into(), value.len().to_string()),
                ];
                HttpResponse {
                    version: "HTTP/1.1".into(),
                    status: 200,
                    reason: "OK".into(),
                    headers,
                    content: value.as_bytes().to_vec(),
                }
            }
            None => HttpResponse::not_found(),
        }
    }

    fn handle_post(&mut self, hostname: String, host_value: String) -> HttpResponse {
        self.hosts_to_hashes.insert(hostname, host_value);
        println!("==== internal state is now: {:?}", self.hosts_to_hashes);
        HttpResponse {
            version: "HTTP/1.1".into(),
            status: 200,
            reason: "OK".into(),
            headers: vec![],
            content: vec![],
        }
    }

    fn handle_delete(&mut self, _hostname: &str) -> HttpResponse {
        // there seem to be security issues here
        // need to store more data than the current model allows to verify
        // that it is indeed the original host requesting to delete their entry
        // and not someone else
        unimplemented!()
    }
}

impl Default for HostnameHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HttpResponse> for Vec<u8> {
    fn from(resp: HttpResponse) -> Self {
        let mut out = String::new();
        let _ = write!(
            &mut out,
            "{} {} {}\r\n",
            resp.version, resp.status, resp.reason
        );

        for (key, val) in resp.headers {
            let _ = write!(out, "{}: {}\r\n", key, val);
        }

        out.push_str("\r\n"); // extra line before body
        let mut out: Vec<u8> = out.into_bytes();
        out.extend(resp.content);
        out.extend("\r\n".as_bytes());

        out
    }
}

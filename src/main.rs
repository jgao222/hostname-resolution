use std::env;
use std::error::Error;
use std::io::{Read, Write, ErrorKind};
use std::net::{TcpListener};
use std::process::exit;

use hostname_resolution_server::*;


fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("Usage: ./hostname_resolution_server [PORT]");
        exit(1)
    }
    let port = args[1].clone();

    let addr = format!("127.0.0.1:{port}");
    // set up server parsing input port
    let listener = TcpListener::bind(addr)?;

    // set up storage
    let mut host_handler = HostnameHandler::new();

    // loop accepting connections forever
    let mut buf: [u8; 256] = [0; 256];
    loop {
        let mut socket = match listener.accept() {
            Ok((socket, addr)) => {
                socket
            }
            Err(e) => {
                eprintln!("{e:?}");
                continue
            }
        };
        let mut request = String::new();
        let mut read_success = true;
        while !request.contains("\r\n\r\n") {
            let bytes_read = match socket.read(&mut buf) {
                Ok(bytes_read) => bytes_read,
                Err(e) => match e.kind() {
                    ErrorKind::Interrupted => continue,
                    _ => { read_success = false; break }
                },
            };
            if bytes_read == 0 {
                read_success = false;
                break;
            }
            request.push_str(&String::from_utf8_lossy(&buf[0..bytes_read]));
        }
        if !read_success {
            continue;
        }
        let (request_heading, request_rest) = request.rsplit_once("\r\n\r\n").unwrap();

        // parse and handle request
        let mut req_parsed = match HttpRequest::try_from(request_heading.to_string()) {
            Err(e) => { eprintln!("{e:?}"); continue },
            Ok(s) => s
        };
        // yep, this feels bad, but it must be done
        // looking for two CRLF in a row only works for requests without bodies (thanks 333 :|)
        if let Some(byte_count) = req_parsed.headers.get("content-length") {
            let byte_count = byte_count.parse().unwrap();
            // keep reading into request_rest until it has at least byte_count bytes
            let mut request_rest = request_rest.to_string();
            while request_rest.len() < byte_count {
                // TODO this code is duplicated from above
                let bytes_read = match socket.read(&mut buf) {
                    Ok(bytes_read) => bytes_read,
                    Err(e) => match e.kind() {
                        ErrorKind::Interrupted => continue,
                        _ => { read_success = false; break }
                    },
                };
                if bytes_read == 0 {
                    read_success = false;
                    break;
                }
                request_rest.push_str(&String::from_utf8_lossy(&buf[0..bytes_read]));
            }
            if !read_success {
                continue;
            }
            let content_bytes = request_rest.as_bytes();
            req_parsed.content.extend_from_slice(&content_bytes[0..byte_count]);
        }
        let res = host_handler.handle_request(&req_parsed);
        let res_bytes: Vec<u8> = res.into();

        match socket.write_all(&res_bytes) {
            Err(e) => { eprintln!("{e:?}"); continue }
            _ => ()
        }
    }

    Ok(())
}

fn escape_escapes(source: &String) -> String {
    source.replace("\n", "\\n").replace("\r", "\\r")
}
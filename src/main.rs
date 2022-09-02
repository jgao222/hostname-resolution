use anyhow::Context;
use std::env;
use std::error::Error;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::process::exit;
use local_ip_address::list_afinet_netifas;

use hostname_resolution_server::*;

#[allow(unreachable_code)] // looping forever is intended
fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("Usage: {} [PORT]", args[0]);
        exit(1)
    }
    let port = args[1].clone();

    // use external crate to get available outwards facing addresses
    let network_interfaces = list_afinet_netifas().unwrap();
    println!("Found these addresses available:");
    for (name, ip) in network_interfaces {
        println!("{}: {}", name, ip);
    }

    // not sure how to try the listed addresses, but apparently 0.0.0.0 requests
    // the OS to host on some address - works on attu, just does localhost on windows
    let addr = format!("0.0.0.0:{port}");
    // set up server parsing input port
    let listener = TcpListener::bind(addr).context("Failed to bind to an address")?;
    println!("Successfully bound to {}", listener.local_addr()?);

    // set up storage
    let mut host_handler = HostnameHandler::new();

    // loop accepting connections forever
    let mut buf: [u8; 256] = [0; 256];
    loop {
        let (mut socket, addr) = match listener.accept() {
            Ok((socket, addr)) => (socket, addr),
            Err(e) => {
                eprintln!("{e:?}");
                continue;
            }
        };
        println!("Accepted Client Connection from {addr}");
        let mut request = String::new();
        let mut read_success = true;
        while !request.contains("\r\n\r\n") {
            let bytes_read = match socket.read(&mut buf) {
                Ok(bytes_read) => bytes_read,
                Err(e) => match e.kind() {
                    ErrorKind::Interrupted => continue,
                    _ => {
                        read_success = false;
                        break;
                    }
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
            Err(e) => {
                eprintln!("{e:?}");
                continue;
            }
            Ok(s) => s,
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
                        _ => {
                            read_success = false;
                            break;
                        }
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
            req_parsed
                .content
                .extend_from_slice(&content_bytes[0..byte_count]);
        }
        let res = host_handler.handle_request(&req_parsed);
        let res_bytes: Vec<u8> = res.into();

        if let Err(e) = socket.write_all(&res_bytes) {
            eprintln!("Error on writing to client: {e:?}");
            continue;
        }
    }

    Ok(())
}

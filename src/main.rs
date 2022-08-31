use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener};

use hostname_resolution_server::*;


fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    println!("args are {:?}", args);

    let port = args[1].clone();

    let addr = format!("127.0.0.1:{port}");
    // set up server parsing input port
    let listener = TcpListener::bind(addr)?;

    // set up storage
    let mut host_handler = HostnameHandler::new();

    // loop accepting connections forever
    loop {
        let mut socket = match listener.accept() {
            Ok((socket, _)) => {
                socket
            }
            Err(e) => {
                eprintln!("{e:?}");
                continue
            }
        };
        let mut request = String::new();
        match socket.read_to_string(&mut request) {
            Ok(bytes) => {
                assert!(request.len() == bytes);
            }
            Err(e) => {
                eprintln!("{e:?}");
                continue;
            }
        }

        // parse and handle request
        let req_parsed = match HttpRequest::try_from(request) {
            Err(e) => { eprintln!("{e:?}"); continue },
            Ok(s) => s
        };
        let res = host_handler.handle_request(&req_parsed);
        let res_bytes: Vec<u8> = res.into();

        match socket.write_all(&res_bytes) {
            Err(e) => { eprintln!("{e:?}"); continue }
            _ => ()
        }
    }

    Ok(())
}
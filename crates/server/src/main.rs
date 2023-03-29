use std::{net::TcpListener, thread::spawn};

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};

use paris::{error, info, success, warn};

fn main() {
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, mut response: Response| {
                info!("Received a new ws handshake");
                Ok(response)
            };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            loop {
                let msg = websocket.read_message().unwrap();
                info!("Received a message: {:?}", msg);
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(msg).unwrap();
                }
            }
        });
    }
}

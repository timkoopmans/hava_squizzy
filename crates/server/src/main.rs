use std::{net::TcpListener, thread::spawn};

use uuid::Uuid;
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
            let uuid = Uuid::new_v4().to_string();
            let callback = |req: &Request, mut response: Response| {
                info!("Opening connection for {}", uuid);
                Ok(response)
            };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            let mut counter = 0;
            let db: sled::Db = sled::open("./db").unwrap();

            loop {
                let msg = websocket.read_message();

                if msg.is_err() {
                    error!("Error reading message: {}", msg.unwrap_err());
                    break;
                }
                let msg = msg.unwrap();
                if msg.is_text() {

                    let key = format!("{}-{}", uuid, counter);
                    let value = msg.to_text().unwrap();
                    info!("Rx: {} {}", key, value);
                    db.insert(key, value).expect("insert failed");
                    counter += 1;
                    websocket.write_message(msg).unwrap();
                } else if msg.is_close() {
                    info!("Closing connection for {}", uuid);
                    websocket.close(None).unwrap();
                    break;
                } else if msg.is_ping() {
                    websocket.write_message(tungstenite::Message::Pong(msg.into_data())).unwrap();
                }
            }
        });
    }
}

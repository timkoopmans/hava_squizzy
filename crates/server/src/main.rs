use std::env;
use std::net::TcpListener;
use std::sync::Arc;
use warp::Filter;

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};
use uuid::Uuid;

use paris::{error, info, success};

#[tokio::main]
async fn main() {
    let server_url = env::var("SERVER_URL").unwrap_or_else(|_| "127.0.0.1:3012".to_string());
    let server = TcpListener::bind(server_url).unwrap();

    let db = Arc::new(sled::open("./db").unwrap());
    serve(&db);

    for stream in server.incoming() {
        let db = db.clone();
        let results_url =
            env::var("RESULTS_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
        tokio::spawn(async move {
            let uuid = Uuid::new_v4().to_string();
            let callback = |_req: &Request, mut response: Response| {
                success!("{}: connection opened", uuid);
                let headers = response.headers_mut();
                headers.append(
                    "x-results-url",
                    format!("{}/results/{}", results_url, uuid).parse().unwrap(),
                );
                Ok(response)
            };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            let mut counter = 0;

            loop {
                let msg = websocket.read_message();

                if msg.is_err() {
                    error!("{}: {}", uuid, msg.unwrap_err());
                    break;
                }
                let msg = msg.unwrap();
                if msg.is_text() {
                    let key = format!("{}-{}", uuid, counter);
                    let value = msg.to_text().unwrap();
                    info!(
                        "{}: {}",
                        key,
                        value
                            .strip_suffix("\r\n")
                            .or(value.strip_suffix("\n"))
                            .unwrap_or(value)
                    );
                    db.insert(key, value).expect("insert failed");
                    db.flush().expect("flush failed");
                    counter += 1;
                    websocket.write_message(msg).unwrap();
                } else if msg.is_close() {
                    info!("{}: connection closed", uuid);
                    websocket.close(None).unwrap();
                    break;
                } else if msg.is_ping() {
                    websocket
                        .write_message(tungstenite::Message::Pong(msg.into_data()))
                        .unwrap();
                }
            }
        });
    }
}

pub fn serve(db: &Arc<sled::Db>) {
    let db = db.clone();
    tokio::spawn(async move {
        let up = warp::path("up").map(|| "have a squizzy taylor");

        let results_by_uuid = warp::path!("results" / String).map(move |uuid| {
            let iter = db.scan_prefix(uuid);
            let results = iter
                .map(|x| String::from_utf8_lossy(&x.unwrap().1).to_string())
                .collect::<Vec<String>>()
                .join("\n");
            format!("{}", results)
        });

        let routes = warp::get().and(up.or(results_by_uuid));

        warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
    });
}

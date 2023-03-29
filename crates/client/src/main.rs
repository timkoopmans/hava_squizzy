use paris::success;
use std::error::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::oneshot;
use tungstenite::{connect, Message};

async fn read_console_output(tx: Sender<String>) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(tokio::io::stdin());
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        tx.send(line).await?;
    }
    Ok(())
}

async fn send_messages(
    url: String,
    mut rx: tokio::sync::mpsc::Receiver<String>,
    stop: oneshot::Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    let (mut socket, response) = connect(url::Url::parse(&*url).unwrap()).expect("Can't connect");

    let url = response.headers().get("x-url").unwrap();

    success!("Results at: {}", url.to_str().unwrap());

    while let Some(message) = rx.recv().await {
        if socket.write_message(Message::text(message).into()).is_err() {
            break;
        }
    }

    let _ = stop.await;
    socket.close(None)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel::<String>(32);
    let (stop_tx, stop_rx) = oneshot::channel();

    let console_task = tokio::spawn(async move {
        read_console_output(tx).await.unwrap();
    });

    let websocket_task = tokio::spawn(async move {
        send_messages("ws://127.0.0.1:3012".to_owned(), rx, stop_rx)
            .await
            .unwrap();
    });

    tokio::select! {
        _ = console_task => {},
        _ = websocket_task => {},
        _ = tokio::signal::ctrl_c() => {},
    }

    stop_tx.send(()).expect("failed to stop");
    Ok(())
}

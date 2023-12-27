use std::time::Duration;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use wrmi::js_types::{JsValue, NullImmediate};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    use futures_util::StreamExt;
    let browser = wrmi::Browser::new();
    let mut outgoing = browser.get_outgoing_stream();
    tokio::spawn({
        let browser = browser.clone();
        async move {
            let document = wrmi::dom::document(&browser);
            let body = document.get_body();
            body.set_inner_text(&"connected!");
            for i in 0..5 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let elem = document.create_element(&"div", &NullImmediate);
                elem.set_inner_text(&&*format!("div: {i}"));
                let txt = document.create_text_node(&&*format!("{i}"));
                body.append_child(&txt);
                body.append_child(&elem);
            }
            wrmi::dom::alert(&browser, &"done");
            wrmi::dom::alert(
                &browser,
                &Into::<JsValue>::into(wrmi::js::Math::exp(&browser, &2.0)),
            );
        }
    });
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(txt))) => {
                        browser.receive_incoming_message(txt);
                    },
                    _ => break
                }
            }
            s = outgoing.next() => {
                match s {
                    Some(s) => {
                        match socket.send(Message::Text(s)).await {
                            Ok(_) => {}
                            Err(_) => break
                        }
                    }
                    _ => break
                }
            }
        }
    }
}
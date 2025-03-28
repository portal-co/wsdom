use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use futures_util::SinkExt;
use wsdom::js_types::JsValue;

#[tokio::main]
async fn main() {
    let router = Router::new().route("/ws", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(app)
}

async fn app(socket: WebSocket) {
    use futures_util::StreamExt;
    let browser = wsdom::Browser::new();

    // make a future that executes our app
    // this is the app part same as in the (non-manual) counter example
    let app_fut = {
        let browser = browser.clone();
        async move {
            let document = wsdom::dom::document(&browser);
            let body = document.get_body();
            let btn_add = document.create_element(&"button", &wsdom::null());
            btn_add.set_inner_text(&"+");
            let btn_sub = document.create_element(&"button", &wsdom::null());
            btn_sub.set_inner_text(&"-");
            let label = document.create_element(&"span", &wsdom::null());
            let mut value = 0;
            let mut click_add = {
                let (callback, func) = wsdom::callback::new_callback::<JsValue>(&browser);
                btn_add.add_event_listener(&"click", &func, &wsdom::null());
                callback
            };
            let mut click_sub = {
                let (callback, func) = wsdom::callback::new_callback::<JsValue>(&browser);
                btn_sub.add_event_listener(&"click", &func, &wsdom::null());
                callback
            };
            body.append_child(&btn_sub);
            body.append_child(&label);
            body.append_child(&btn_add);
            loop {
                label.set_inner_text(&&*format!("{value}"));
                tokio::select! {
                    _ = click_add.next() => {
                        value += 1;
                    }
                    _ = click_sub.next() => {
                        value -= 1;
                    }
                }
            }
        }
    };

    // split the WebSocket into a sender and a receiver
    let (mut tx, mut rx) = socket.split();

    // make a Future that takes message from WSDOM and forward it into the WebSocket sender
    let tx_fut = {
        let browser = browser.clone();
        async move {
            tx.send_all(&mut browser.map(|msg| Ok(Message::Text(msg))))
                .await
        }
    };

    // make a Future that takes message from the WebSocket receiver and forward it to WSDOM
    let rx_fut = {
        let browser = browser.clone();
        async move {
            while let Some(Ok(Message::Text(msg))) = rx.next().await {
                browser.receive_incoming_message(msg);
            }
        }
    };

    // execute all threee futures at once
    let _todo = tokio::join!(tx_fut, rx_fut, app_fut);
}

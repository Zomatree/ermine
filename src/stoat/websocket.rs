use futures::{FutureExt, SinkExt, StreamExt, future::select};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        Mutex,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
    task::AbortHandle,
    time::sleep,
};
use tokio_tungstenite::connect_async_with_config;
use tungstenite::{Message, protocol::WebSocketConfig};

use crate::{
    error::Error,
    http,
    types::{ClientMessage, EventV1, Ping},
};

#[derive(Debug, Clone)]
pub enum LocalEvent {
    Disconnected,
    Reconnecting,
    Reconnected,
    Connected,
}

#[derive(Debug, Clone)]
pub enum Event {
    Stoat(EventV1),
    Local(LocalEvent),
}

async fn send(
    ws: &Arc<Mutex<impl SinkExt<Message, Error = tungstenite::Error> + Unpin>>,
    event: &ClientMessage,
) -> Result<(), tungstenite::Error> {
    let mut lock = ws.lock().await;

    let message = Message::text(serde_json::to_string(event).unwrap());

    lock.send(message).await
}

pub async fn run(
    events: UnboundedSender<Event>,
    client_events: Arc<Mutex<UnboundedReceiver<ClientMessage>>>,
) -> Result<(), Error> {
    let http = http();

    let uri = format!(
        "{}/?token={}&format=json&ready=users&ready=servers&ready=channels&ready=members&ready=channel_unreads&ready=emojis",
        &http.api_config.ws,
        http.session.read().unwrap().as_ref().expect("No session").token.clone()
    );

    log::debug!("Connecting to websocket with {uri}");

    let mut ws_config = WebSocketConfig::default();
    ws_config.max_frame_size = Some(usize::MAX);
    ws_config.max_message_size = Some(usize::MAX);

    let (ws, _) = connect_async_with_config(uri, Some(ws_config), false)
        .await
        .inspect_err(|e| {
            if let tungstenite::Error::Http(resp) = e
                && let Some(body) = resp.body()
                && let Ok(body) = std::str::from_utf8(body)
            {
                log::error!("Error when attempting to establish websocket connection:\n{body}");
            };
        })?;

    let (ws_send, mut ws_receive) = ws.split();

    let ws_send = Arc::new(Mutex::new(ws_send));

    let server_client = {
        let ws_send = ws_send.clone();

        async move {
            let mut task: Option<AbortHandle> = None;

            while let Some(msg) = ws_receive.next().await {
                let msg = msg?;

                let event = match msg {
                    Message::Text(data) => {
                        serde_json::from_str(data.as_str()).map_err(|e| e.to_string())
                    }
                    Message::Close(_) => return Err(Error::ClosedWs),
                    msg => {
                        if let Ok(text) = msg.to_text() {
                            log::error!("Unexpected WS message: {text:?}");
                        } else {
                            log::error!("Unexpected WS message: {:?}", msg.into_data());
                        }
                        continue;
                    }
                };

                match event {
                    Ok(event) => {
                        log::debug!("Received event {event:?}");

                        if let EventV1::Authenticated = &event {
                            task = Some(
                                tokio::spawn({
                                    let ws = ws_send.clone();
                                    let mut i = 0;

                                    async move {
                                        loop {
                                            // sleep(Duration::from_secs(5)).await;
                                            // ws.lock().await.close().await;

                                            if send(
                                                &ws,
                                                &ClientMessage::Ping {
                                                    data: Ping::Number(i),
                                                },
                                            )
                                            .await
                                            .is_err()
                                            {
                                                return;
                                            };

                                            i = i.wrapping_add(1);

                                            sleep(Duration::from_secs(30)).await;
                                        }
                                    }
                                })
                                .abort_handle(),
                            );
                        };

                        events
                            .send(Event::Stoat(event))
                            .map_err(|_| Error::InternalError)?;
                    }
                    Err(e) => {
                        log::error!("Failed to deserialise event: {e:?}");
                    }
                }
            }

            if let Some(task) = task {
                task.abort();
            };

            Err(Error::ClosedWs)
        }
    }
    .boxed();

    let client_server = {
        let ws_send = ws_send.clone();

        async move {
            while let Some(message) = client_events.lock().await.recv().await {
                send(&ws_send, &message).await?
            }

            Err(Error::ClosedWsLocal)
        }
    }
    .boxed();

    select(server_client, client_server).await.into_inner().0
}

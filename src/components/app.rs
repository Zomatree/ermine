use std::{collections::HashMap, sync::Arc, time::Duration};

use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use serde_json::to_value;
use stoat_models::v0;
use tokio::{
    sync::{Mutex, mpsc},
    time::sleep,
};

use crate::{
    AppChannel, ConnectionState, SettingsState, components, consume_material_theme, http, state,
    use_changed_with_previous, use_config,
    websocket::{self, Event, LocalEvent},
};

#[derive(PartialEq)]
pub struct App {}

impl Component for App {
    fn render(&self) -> impl IntoElement {
        let station = use_radio_station();
        let config = use_config();
        let theme = consume_material_theme();

        let (_msg_s, msg_r) = use_hook(|| {
            let (s, r) = mpsc::unbounded_channel();

            (provide_context(s), Arc::new(Mutex::new(r)))
        });

        let (event_s, event_r) = use_hook(|| {
            let (s, r) = mpsc::unbounded_channel();

            (s, Arc::new(Mutex::new(r)))
        });

        let ws = use_state(move || {
            let event_s = event_s.clone();
            let msg_r = msg_r.clone();

            tokio::spawn(async move {
                loop {
                    if let Err(e) = websocket::run(event_s.clone(), msg_r.clone()).await {
                        println!("{e:?}");
                    };

                    event_s
                        .send(Event::Local(LocalEvent::Disconnected))
                        .unwrap();
                    sleep(Duration::from_secs(1)).await;
                    event_s
                        .send(Event::Local(LocalEvent::Reconnecting))
                        .unwrap();
                }
            })
            .abort_handle()
        });

        use_drop(move || ws.read().abort());

        use_future(move || {
            let event_r = event_r.clone();
            let mut station = station.clone();

            async move {
                while let Some(event) = event_r.lock().await.recv().await {
                    log::debug!("{event:?}");

                    match event {
                        Event::Stoat(event) => state::update_state(event, config, station).await,
                        Event::Local(event) => {
                            station.write_channel(AppChannel::State).state = match event {
                                LocalEvent::Disconnected => ConnectionState::Disconnected,
                                LocalEvent::Reconnecting => ConnectionState::Reconnecting,
                                LocalEvent::Reconnected => ConnectionState::Reconnected,
                                LocalEvent::Connected => ConnectionState::Connected,
                            }
                        }
                    }
                }
            }
        });

        let radio = use_radio(AppChannel::Ready);

        use_future(move || {
            let mut radio = radio.clone();

            async move {
                if let Ok(settings) = http()
                    .fetch_settings(&v0::OptionsFetchSettings {
                        keys: vec![
                            "ordering".to_string(),
                            "notifications".to_string(),
                            "ermine".to_string(),
                        ],
                    })
                    .await
                {
                    let mut state = radio.write_silently();

                    for (key, (_ts, payload)) in settings.into_iter() {
                        match key.as_str() {
                            "ordering" => {
                                if let Ok(value) = serde_json::from_str(&payload) {
                                    state.settings.ordering = Some(value)
                                }
                            }
                            "notifications" => {
                                if let Ok(value) =
                                    Ok::<_, ()>(serde_json::from_str(&payload).unwrap())
                                {
                                    state.settings.notifications = Some(value)
                                }
                            }
                            "ermine" => {
                                if let Ok(value) =
                                    Ok::<_, ()>(serde_json::from_str(&payload).unwrap())
                                {
                                    state.settings.ermine = Some(value)
                                }
                            }
                            _ => {}
                        }
                    }
                }

                radio.write().ready.settings = true;
            }
        });

        let settings = radio.slice(AppChannel::Settings(""), |state| &state.settings);
        let mut update_settings_task = use_state(|| None::<TaskHandle>);

        use_changed_with_previous::<SettingsState>(settings, move |before, settings| {
            let cancel = {
                let task = update_settings_task.peek();
                task.as_ref().map(|t| t.cancel());
                task.is_none()
            };

            let settings = settings.clone();

            update_settings_task.set(Some(spawn(async move {
                if cancel {
                    return;
                };

                sleep(Duration::from_secs(5)).await;

                let mut map = HashMap::new();

                if &before.ordering != &settings.ordering
                    && let Some(ordering) = &settings.ordering
                {
                    map.insert("ordering".to_string(), to_value(ordering).unwrap());
                }

                if &before.notifications != &settings.notifications
                    && let Some(notifications) = &settings.notifications
                {
                    map.insert(
                        "notifications".to_string(),
                        to_value(notifications).unwrap(),
                    );
                }

                if &before.ermine != &settings.ermine
                    && let Some(ermine) = &settings.ermine
                {
                    map.insert("ermine".to_string(), to_value(ermine).unwrap());
                }
                http().set_settings(&map).await.unwrap();
            })));
        });

        if radio.read().ready.is_ready() {
            components::Client {}.into_element()
        } else {
            rect()
                .width(Size::Fill)
                .height(Size::Fill)
                .center()
                .child(CircularLoader::new().primary_color(theme.md.on_surface.as_argb_u32()))
                .into_element()
        }
    }
}

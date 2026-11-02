use std::{collections::HashMap, time::Duration};

use freya::{prelude::*, radio::use_radio};
use serde_json::to_value;
use stoat_models::v0;
use tokio::time::sleep;

use crate::{AppChannel, SettingsState, http, use_changed_with_previous};

pub fn use_init_settings() {
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
                        "server-folders".to_string(),
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
                            if let Ok(value) = Ok::<_, ()>(serde_json::from_str(&payload).unwrap())
                            {
                                state.settings.notifications = Some(value)
                            }
                        }
                        "ermine" => {
                            if let Ok(value) = Ok::<_, ()>(serde_json::from_str(&payload).unwrap())
                            {
                                state.settings.ermine = Some(value)
                            }
                        }
                        "server-folders" => {
                            if let Ok(value) = Ok::<_, ()>(serde_json::from_str(&payload).unwrap())
                            {
                                state.settings.server_folders = Some(value)
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

            if &before.server_folders != &settings.server_folders
                && let Some(server_folders) = &settings.server_folders
            {
                map.insert(
                    "server-folders".to_string(),
                    to_value(server_folders).unwrap(),
                );
            }

            http().set_settings(&map).await.unwrap();
        })));
    });
}

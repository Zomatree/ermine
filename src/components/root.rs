use freya::prelude::*;

use crate::{
    Config,
    components::{App, Login},
    http,
};

#[derive(PartialEq)]
pub struct Root {}

impl Component for Root {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();

        let session = use_reactive(&config.read().session);

        use_side_effect(move || {
            {
                let session = session.read();
                *http().session.write().unwrap() = session.clone();
            };

            config.write();
        });

        if config.read().session.is_some() && http().session.read().unwrap().is_some() {
            App {}.into_element()
        } else {
            Login {}.into_element()
        }
    }
}

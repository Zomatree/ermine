use freya::prelude::*;

pub fn use_clipboard() -> State<arboard::Clipboard> {
    use_hook(|| match try_consume_root_context() {
        Some(state) => state,
        None => {
            let state = State::create_global(
                arboard::Clipboard::new().expect("Failed to connect to clipboard"),
            );
            provide_root_context(state);
            state
        }
    })
}

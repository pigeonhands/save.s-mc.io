use frontend::App;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default().module_prefix("frontend"));

    mount_to_body(|| {
        view! {
            <App />
        }
    })
}

mod app;
mod components;
mod gpg;
mod routes;

use app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    yew::Renderer::<App>::new().render();
}

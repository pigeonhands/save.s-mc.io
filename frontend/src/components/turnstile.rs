// Turnstile
use leptos::prelude::*;
use wasm_bindgen::{convert::IntoWasmAbi, prelude::*};
use web_sys::{
    Element,
    js_sys::{self, JsString, Reflect},
    wasm_bindgen::prelude::Closure,
};

const TURNSTILE_SITE_KEY: Option<&'static str> = option_env!("TURNSTILE_SITE_KEY");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = TurnstileJS)]
    type TurnstileJS;

    #[wasm_bindgen(method, js_name = render)]
    fn render(this: &TurnstileJS, node: JsString, args: js_sys::Object) -> u32;

    #[wasm_bindgen(method, js_name = reset)]
    fn reset(this: &TurnstileJS, node: JsString) -> u32;

    #[wasm_bindgen(method, js_name = getResponse)]
    fn get_response(this: &TurnstileJS) -> JsString;

}

/// A parameterized incrementing button
#[component]
pub fn Turnstile() -> impl IntoView {
    view! {
        <div
        class="cf-turnstile"
        ></div>
    }
}
fn get_obj() -> Option<TurnstileJS> {
    let t = gloo::utils::window().get("turnstile")?;
    let t_js = t.unchecked_into::<TurnstileJS>();
    Some(t_js)
}

pub fn enabled() -> bool {
    TURNSTILE_SITE_KEY.is_some()
}

pub fn response() -> Option<String> {
    if !enabled() {
        return None;
    }
    let t_js = get_obj()?;
    Some(t_js.get_response().into())
}

pub fn render() -> bool {
    let key = match TURNSTILE_SITE_KEY {
        Some(k) => k,
        None => {
            log::warn!("No cf site key set");
            return false;
        }
    };

    let t_js = match get_obj() {
        Some(obj) => obj,
        None => {
            log::warn!("No turnstile object on window");
            return false;
        }
    };

    let callback: Closure<dyn FnMut(JsValue)> = Closure::new(move |value: JsValue| {
        if let Some(s) = value.as_string() {
            log::debug!("Turnsible key: {:?}", s);
        }
    });

    let error_callback: Closure<dyn FnMut(JsValue)> = Closure::new(move |value: JsValue| {
        log::error!("Turnstile error {:?}", &value);
    });

    let args = js_sys::Object::new();

    Reflect::set(&args, &JsValue::from("sitekey"), &JsValue::from(key)).unwrap();
    Reflect::set(&args, &JsValue::from("callback"), &callback.into_js_value()).unwrap();
    Reflect::set(
        &args,
        &JsValue::from("error-callback"),
        &error_callback.into_js_value(),
    )
    .unwrap();

    t_js.render(".cf-turnstile".into(), args);

    true
}

pub fn reset() -> bool {
    let t_js = match get_obj() {
        Some(obj) => obj,
        None => {
            log::warn!("No turnstile object on window");
            return false;
        }
    };

    t_js.reset(".cf-turnstile".into());
    true
}

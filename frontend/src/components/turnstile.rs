use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use wasm_bindgen::{convert::IntoWasmAbi, prelude::*};
use web_sys::{
    Element,
    js_sys::{self, JsString, Reflect},
    wasm_bindgen::prelude::Closure,
};
use yew::prelude::*;

const TURNSTILE_SITE_KEY: Option<&'static str> = option_env!("TURNSTILE_SITE_KEY");

#[derive(Clone)]
pub struct Turnstile {
    pub cb: Rc<RefCell<Option<Box<dyn FnMut(String)>>>>,
}

impl Turnstile {
    pub fn new(cb: Box<dyn FnMut(String)>) -> Self {
        let elem: web_sys::Element = gloo::utils::document().create_element("div").unwrap();
        elem.set_class_name("cf-turnstile");
        Self {
            cb: Rc::from(RefCell::from(Some(cb))),
        }
    }

    pub fn enabled(&self) -> bool {
        TURNSTILE_SITE_KEY.is_some()
    }

    pub fn elem(&self) -> Html {
        if self.enabled() {
            html! {
            <div
            class="cf-turnstile"
            ></div>
            }
        } else {
            html! {}
        }
    }

    fn get_obj(&self) -> Option<TurnstileJS> {
        let t = gloo::utils::window().get("turnstile")?;
        let t_js = t.unchecked_into::<TurnstileJS>();
        Some(t_js)
    }

    pub fn response(&self) -> Option<String> {
        if !self.enabled() {
            return None;
        }
        let t_js = self.get_obj()?;
        Some(t_js.get_response().into())
    }

    pub fn reset(&self) -> bool {
        let t_js = match self.get_obj() {
            Some(obj) => obj,
            None => {
                log::warn!("No turnstile object on window");
                return false;
            }
        };

        t_js.reset(".cf-turnstile".into());
        return true;
    }

    pub fn render(&self) -> bool {
        let mut cb = match self.cb.take() {
            Some(cb) => cb,
            None => {
                log::warn!("No callback set");
                return false;
            }
        };
        let key = match TURNSTILE_SITE_KEY {
            Some(k) => k,
            None => {
                log::warn!("No cf site key set");
                return false;
            }
        };

        let t_js = match self.get_obj() {
            Some(obj) => obj,
            None => {
                log::warn!("No turnstile object on window");
                return false;
            }
        };

        let callback: Closure<dyn FnMut(JsValue)> = Closure::new(move |value: JsValue| {
            log::debug!("Turnstile callback {:?}", &value);
            if let Some(s) = value.as_string() {
                cb(s);
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

        false
    }
}

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

use crate::components::turnstile;
use closure::closure;
use common::{PublicKeyRequest, PublicKeyResponse};
use gloo_net::http::Request;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_hooks::use_mount;

#[function_component(Send)]
pub fn send() -> Html {
    html! {
        <div box_tui="square" shear_tui="top" class="h-full">
            <div class="header" >
                <span class="box-title">
                    <h1>{ "Save" }</h1>
                </span>
            </div>

            <SendForm />
        </div>
    }
}

#[derive(Debug, Clone)]
pub struct SendFormState {
    pub email: UseStateHandle<AttrValue>,
    pub description: UseStateHandle<AttrValue>,
    pub message: UseStateHandle<AttrValue>,
    pub encrypted_message: UseStateHandle<Option<AttrValue>>,
}

#[function_component(SendForm)]
pub fn send_form() -> Html {
    let cf_turnstile = turnstile::Turnstile::new(Box::new(move |a| {
        log::debug!("CF callback {}", a);
    }));

    use_mount(closure!(clone cf_turnstile, || {
        if cf_turnstile.enabled() {
            cf_turnstile.render();
        }
    }));

    let show_validation = use_state(|| false);
    let form = SendFormState {
        email: use_state(AttrValue::default),
        description: use_state(AttrValue::default),
        message: use_state(AttrValue::default),
        encrypted_message: use_state(|| None),
    };

    let onchange_email = {
        Callback::from(closure!(clone form.email, |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            email.set(input.value().into());
        }))
    };

    let onchange_description = {
        Callback::from(closure!(clone form.description, |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            description.set(input.value().into());
        }))
    };

    let onchange_message = {
        Callback::from(closure!(clone form.message, |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            message.set(input.value().into());
        }))
    };

    let on_edit_message = {
        Callback::from(closure!(clone form.encrypted_message, |_| {
            encrypted_message.set(None);
        }))
    };

    let on_encrypt = {
        let form = form.clone();
        let cf_turnstile = cf_turnstile.clone();
        let show_validation = show_validation.clone();

        Callback::from(move |_| {
            show_validation.set(true);

            let form = form.clone();
            let cf_turnstile = cf_turnstile.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let resp = Request::get("/public-key")
                    .header(
                        "Captcha-Response",
                        cf_turnstile
                            .response()
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or(""),
                    )
                    .query(
                        PublicKeyRequest {
                            email: form.email.to_string(),
                        }
                        .iter(),
                    )
                    .send()
                    .await
                    .unwrap();

                let resp_json: PublicKeyResponse = resp.json().await.unwrap();

                match crate::gpg::encrypt(
                    resp_json.pub_key,
                    (*form.description).as_str().into(),
                    (*form.message).as_str().into(),
                ) {
                    Ok(encrypted) => {
                        form.encrypted_message.set(Some(encrypted.into()));
                    }
                    Err(e) => {
                        log::error!("gpg encryption error: {}", e);
                    }
                }
            });
        })
    };

    let validation_classes = |state_handle: UseStateHandle<AttrValue>| {
        log::info!(
            "validating. {:?} - {}",
            state_handle.clone(),
            (*state_handle).clone().is_empty()
        );
        if *show_validation && (*state_handle).is_empty() {
            "invalid-input"
        } else {
            ""
        }
    };

    let message_display = || {
        if let Some(m) = (*form.encrypted_message).clone() {
            m
        } else {
            (*form.message).clone()
        }
    };

    html! {

            <div class="p-5 flex flex-col items-center h-full" gap_tui="0">

                <div class="grid grid-cols-2 justify-between">
                    <div box_tui="round" shear_tui="top" class={validation_classes(form.email.clone())}>
                        <span class="box-title">{ "Email" }</span>
                        <input
                            name="email"
                            class="h-11 w-full"
                            style="background: var(--background)"
                            onchange={onchange_email}
                        />
                    </div>

                    <div box_tui="round" shear_tui="top" class={validation_classes(form.description.clone())}
    >
                        <span class="box-title">{ "Description" }</span>
                        <input
                            name="email"
                            class="h-11 w-full"
                            style="background: var(--background)"
                            onchange={onchange_description}
                        />
                    </div>
                </div>

                <div box_tui="round" shear_tui="top" class={classes!("w-full", "h-full", validation_classes(form.message.clone()))}>
                    <div class="header">
                        <span class="box-title">{ "Message" }</span>
                        <span>
                            <span is_tui="badge" class="mr-[2ch]">{ "Text" }</span>
                            <span is_tui="badge" variant_tui="background1" class="mr-[2ch]">{ "File" }</span>
                        </span>
                    </div>
                    <textarea
                            value={message_display()}
                            onchange={onchange_message}
                            //disabled={encrypting || encrypted}
                            name="message"
                            class="w-full h-95/100 resize-none"
                            size_tui="large" placeholder="..."
                            style="background: var(--background)"></textarea>
                </div>

                {cf_turnstile.elem()}

                if form.encrypted_message.is_some() {
                    <button
                        type="button"
                        onclick={on_edit_message}
                        variant_tui="foreground1"
                        id="login-button">{ "Edit message" }
                    </button>
                } else {
                    <button
                        type="button"
                        onclick={on_encrypt}
                        variant_tui="foreground1"
                        id="login-button">{ "Encrypt" }
                    </button>
                }
            </div>

        }
}

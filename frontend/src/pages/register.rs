use crate::{components::turnstile, providers::api};
use leptos::prelude::*;

#[component]
pub fn Register() -> impl IntoView {
    view! {
        <div class="h-full" box-="round" shear-="top">
            <div class="header" >
                <span class="box-title">
                    <h1> "Register" </h1>
                </span>
            </div>

            <RegisterForm />
        </div>
    }
}

#[component]
pub fn RegisterForm() -> impl IntoView {
    Effect::new(move |_| {
        if turnstile::enabled() {
            if !turnstile::render() {
                log::error!("Failed to render turnstile");
            }
        }
    });

    view! {
        <div class="p-5 flex flex-col items-center h-full" gap-="0">

            <div box-="round" shear-="top" class="w-full h-full" >
                <div class="header">
                    <span class="box-title">PGP Public Key</span>
                </div>
                <textarea
                        name="message"
                        class="w-full h-95/100 resize-none"
                        size-="large" placeholder="..."
                        style="background: var(--background)"
                        required
                ></textarea>

            </div>



            <div class="grid grid-cols-1">
                <button
                    variant-="foreground1"
                    id="login-button"
                    type="button"
                >
                    Register
                </button>


            </div>

                { move || if turnstile::enabled() {
                    view! {
                        <turnstile::Turnstile />
                    }.into_any()
                    }else { view!{
                        <></>
                    }.into_any() }
                }

            </div>

    }
}

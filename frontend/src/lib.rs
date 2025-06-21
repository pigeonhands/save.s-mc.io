pub use leptos;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::components::turnstile;

// Modules
mod components;
mod pages;
mod providers;

// Top-Level pages
use crate::pages::home::Home;
use crate::pages::register::Register;
use crate::pages::save::Save;

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

        // sets the document title
        <Title text="Save things for later securely" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Layout>
            <Router>
                <Routes fallback=|| view! { NotFound }>
                    <Route path=path!("/") view=Save />
                    <Route path=path!("/register") view=Register />
                </Routes>
            </Router>
        </Layout>
    }
}
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    #[cfg(feature = "turnstile")]
    Effect::new(move |_| {
        if turnstile::enabled() {
            if !turnstile::render() {
                log::error!("Failed to render turnstile");
            }
        }
    });

    view! {
        <div class="grid grid-rows-[auto_1fr_auto] h-lvh" shear-="top">
            <header class="header" >
                <span>
                <h1 class="box-title">{ "save.s-mc.io" }</h1>
                </span>
                <span>
                    <span class="mr-[2ch]"><a class="link" href="/">{ "save" }</a></span>
                    <span class="mr-[2ch]"><a class="link" href="/read">{ "read" }</a></span>
                    <span class="mr-[2ch]"><a class="link" href="/register">{ "register" }</a></span>
                </span>
            </header>

            <div class="container mx-auto grid grid-cols-1 xl:grid-cols-[200px_minmax(0px,_1fr)_200px]">
                <aside class="sticky top-0 col-span-1 hidden  p-4 xl:block">
                </aside>
                <div class="col-span-1 space-y-4 p-4 ">
                    { children() }
                </div>
                <aside class="sticky top-0 col-span-1 hidden  p-4 xl:block">
                </aside>
            </div>
            <footer>

                <span class="container mx-auto flex justify-center items-center my-5" style="color: grey">
                    { move || if turnstile::enabled() {
                        view! {
                            <turnstile::Turnstile />
                        }.into_any()
                        }else { view!{
                            <></>
                        }.into_any() }
                    }
                    <a href="mailto:contact@s-mc.io">{ "contact@s-mc.io" }</a><span class="mx-2">{ "-" }</span><a href="/pgp.txt">{ "pgp" }</a>
                </span>
            </footer>
        </div>
    }
}

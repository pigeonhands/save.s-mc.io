use crate::routes;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main style="display: contents">
            <Layout>
            <BrowserRouter>
                        <Switch<routes::Route> render={routes::switch} />
                </BrowserRouter>
            </Layout>
        </main>
    }
}

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Html,
}

#[function_component(Layout)]
fn layout(props: &LayoutProps) -> Html {
    html! {
        <div class="grid grid-rows-[auto_1fr_auto] h-lvh" shear_tui="top">
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
                    { props.children.clone() } // you can forward children like this
                </div>
                <aside class="sticky top-0 col-span-1 hidden  p-4 xl:block">
                </aside>
            </div>
            <footer>
                <span class="container mx-auto flex justify-center items-center my-5" style="color: grey">
                    <a href="mailto:contact@s-mc.io">{ "contact@s-mc.io" }</a><span class="mx-2">{ "-" }</span><a href="/pgp.txt">{ "pgp" }</a>
                </span>
            </footer>
        </div>
    }
}

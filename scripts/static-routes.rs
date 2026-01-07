#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
frontend = { version = "*", path = "../frontend", default-features = false }
leptos_axum = { version = "0.8.2", default-features = false }
---


fn main() {
    let dist = std::path::Path::new("./dist/assets/");
    let index = dist.join("index.html");

    std::fs::copy(&index, dist.join("404.html"));

    let routes = leptos_axum::generate_route_list(frontend::App);
    for route in routes {
        let stripped = route.path().strip_prefix("/").unwrap();
        if stripped.is_empty() {
            continue;
        }
        let new_route = dist.join(format!("{}.html", stripped));
        println!("Route: {} - {}", route.path(), new_route.display());

        std::fs::copy(&index, &new_route).unwrap();
    }
}

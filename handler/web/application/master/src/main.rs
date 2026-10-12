//! Master Application Entrypoint (Leptos CSR WASM SPA)

mod api_client;
mod app;
mod access;
mod audit;
mod layouts;
mod pages;
mod routes;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

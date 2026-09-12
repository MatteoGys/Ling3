mod api;
mod app;
mod icons;
mod theme;
mod views;

use app::App;
use leptos::mount::mount_to_body;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| leptos::view! { <App /> });
}

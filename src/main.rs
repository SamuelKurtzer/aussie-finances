mod app;
mod components;
mod domain;
mod pages;

fn main() {
    leptos::mount_to_body(app::App);
}

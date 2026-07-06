mod app;
mod components;
mod domain;
mod formatting;
mod pages;
mod storage;

fn main() {
    leptos::mount_to_body(app::App);
}

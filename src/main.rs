#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::non_ascii_literal,
    clippy::multiple_crate_versions,
    clippy::module_name_repetitions,
    clippy::else_if_without_else
)]

mod application;
mod components;
mod util;

pub use components::canteen;

use gettextrs::TextDomain;

fn main() {
    let mut domain = TextDomain::new("gnome-ovgu-canteen").codeset("UTF-8");
    if let Ok(xdg) = xdg::BaseDirectories::new() {
        domain = domain.prepend(xdg.get_data_home());
    }
    domain.init()
        .expect("Failed to initialize translation domain");

    match application::Application::new() {
        Ok(app) => {
            std::process::exit(app.run(&std::env::args().collect::<Vec<_>>()));
        }
        Err(msg) => {
            eprintln!("error: {}", msg);
            std::process::exit(1337);
        }
    };
}

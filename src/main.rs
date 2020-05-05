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
use flexi_logger::{colored_with_thread, Logger};

fn main() {
    Logger::with_env_or_str("warn, ovgu_canteen_gtk=info")
        .format(colored_with_thread)
        .start()
        .expect("logger initialization failed");

    let mut domain = TextDomain::new("ovgu-canteen-gtk").codeset("UTF-8");
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
            log::error!("error: {}", msg);
            std::process::exit(1337);
        }
    };
}

mod badge;
pub mod canteen;
mod day;
mod meal;
mod window;
pub mod preferences;

pub use badge::{BadgeComponent, LiteBadgeComponent, SymbolComponent};
pub use canteen::CanteenComponent;
pub use day::DayComponent;
pub use meal::MealComponent;
pub use window::WindowComponent;

// the content of the glade file
pub const GLADE: &str = std::include_str!("../../data/io.github.fin_ger.OvGUCanteen.glade");

// enables us to yield execution when running in the UI thread to GTK for UI updating
// this makes the application more responsive on slow PCs
macro_rules! glib_yield {
    () => {
        glib::timeout_future_with_priority(glib::PRIORITY_DEFAULT_IDLE, 0).await
    };
}
pub(crate) use glib_yield;

// a shortcut for getting an object-id from a GTK Builder with a helpful error context
macro_rules! get {
    ($builder:expr, $id:expr) => {{
        use anyhow::Context;
        use gtk::prelude::*;

        let builder = $builder;
        let id = $id;
        builder.get_object(id).context(format!(
            "'{}' is not available in glade file: {}:{}:{}",
            id,
            file!(),
            line!(),
            column!()
        ))
    }};
}
pub(crate) use get;

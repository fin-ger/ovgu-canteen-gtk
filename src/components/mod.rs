mod badge;
pub mod canteen;
mod day;
mod meal;
mod window;
pub mod preferences;

pub use badge::{BadgeComponent, LiteBadgeComponent};
pub use canteen::CanteenComponent;
pub use day::DayComponent;
pub use meal::MealComponent;
pub use window::WindowComponent;

pub const GLADE: &str = std::include_str!("../../data/de.fin_ger.OvGUCanteen.glade");

macro_rules! glib_yield {
    () => {
        glib::timeout_future_with_priority(glib::PRIORITY_DEFAULT_IDLE, 0).await
    };
}
pub(crate) use glib_yield;

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

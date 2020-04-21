mod badge;
mod canteen;
mod day;
mod meal;
mod window;

use anyhow::{Context, Result};
use gtk::prelude::*;
use gtk::Builder;

pub use badge::{BadgeComponent, LiteBadgeComponent};
pub use canteen::CanteenComponent;
pub use day::DayComponent;
pub use meal::MealComponent;
pub use window::WindowComponent;

pub const GLADE: &str = std::include_str!("../../data/gnome-ovgu-canteen.glade");

macro_rules! glib_yield {
    () => {
        glib::timeout_future_with_priority(glib::PRIORITY_DEFAULT_IDLE, 0).await
    };
}

pub(crate) use glib_yield;

#[inline]
pub fn get<T: IsA<glib::Object>>(builder: &Builder, id: &str) -> Result<T> {
    builder
        .get_object(id)
        .context(format!("'{}' is not available in glade file", id))
}

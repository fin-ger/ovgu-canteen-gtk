use anyhow::{Error, Result};
use gtk::prelude::*;
use gtk::{Box, Builder, Label, Spinner, Stack, ScrolledWindow};
use gettextrs::gettext as t;
use ovgu_canteen::{Canteen, CanteenDescription};

use crate::components::{get, glib_yield, DayComponent, WindowComponent, GLADE};
use crate::util::{enclose, AdjustingVec};

#[derive(Debug)]
pub struct CanteenComponent {
    canteen_stack: Stack,
    canteen_scrolled_window: ScrolledWindow,
    canteen_error_label: Label,
    canteen_spinner: Spinner,
    days: AdjustingVec<DayComponent, Error>,
}

pub fn translate(description: &CanteenDescription) -> String {
    match description {
        CanteenDescription::UniCampusLowerHall => t("UniCampus Magdeburg Lower Hall"),
        CanteenDescription::UniCampusUpperHall => t("UniCampus Magdeburg Upper Hall"),
        CanteenDescription::Kellercafe => t("Kellercafé Magdeburg"),
        CanteenDescription::Herrenkrug => t("Herrenkrug Magdeburg"),
        CanteenDescription::Stendal => t("Stendal"),
        CanteenDescription::Wernigerode => t("Wernigerode"),
        CanteenDescription::DomCafeteHalberstadt => t("DomCafete Halberstadt"),
    }
}

impl CanteenComponent {
    pub fn new(description: &CanteenDescription, window: &WindowComponent) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let canteen_stack: Stack = get!(&builder, "canteen-stack")?;
        let canteen_scrolled_window: ScrolledWindow = get!(&builder, "canteen-scrolled-window")?;
        let canteen_error_label: Label = get!(&builder, "canteen-error-label")?;
        let canteen_spinner: Spinner = get!(&builder, "canteen-spinner")?;
        let days_box: Box = get!(&builder, "days-box")?;
        let canteen_name = translate(description);

        window.add_canteen(&canteen_stack, serde_plain::to_string(description).unwrap(), canteen_name)?;

        let days = AdjustingVec::new(
            enclose! { (canteen_scrolled_window, days_box) move || {
                enclose! { (canteen_scrolled_window, days_box) async move {
                    let comp = DayComponent::new(move |y| {
                        Self::scroll_to(&canteen_scrolled_window, y);
                    }).await?;
                    days_box.pack_start(comp.root_widget(), false, true, 0);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            |day| async move {
                day.root_widget().destroy();
                glib_yield!();
                Ok(())
            },
        );

        Ok(Self {
            canteen_stack,
            canteen_scrolled_window,
            canteen_error_label,
            canteen_spinner,
            days,
        })
    }

    fn scroll_to(canteen_scrolled_window: &ScrolledWindow, y: i32) {
        if let Some(position) = canteen_scrolled_window.get_vadjustment() {
            position.set_value(y as f64 - 42.0);
            canteen_scrolled_window.set_vadjustment(Some(&position));
        }
    }

    pub async fn load(&mut self, load_result: Result<Canteen>) -> Option<Canteen> {
        self.canteen_spinner.start();
        self.canteen_spinner.show();
        self.canteen_stack.set_visible_child_name("canteen-menu");

        let canteen = match load_result {
            Ok(canteen) => {
                self.canteen_stack.set_visible_child_name("canteen-menu");
                canteen
            }
            Err(e) => {
                self.canteen_stack.set_visible_child_name("canteen-error");
                self.canteen_error_label
                    .set_text(&format!("{}: {:#}", t("error"), e));
                return None;
            }
        };

        let days_result = self
            .days
            .adjust(&canteen.days, |mut comp, day| async move {
                comp.load(day).await;
                glib_yield!();
                Ok(comp)
            })
            .await;

        if let Err(e) = days_result {
            self.canteen_stack.set_visible_child_name("canteen-error");
            self.canteen_error_label
                .set_text(&format!("{}: {:#}", t("error"), e));
        } else if canteen.days.is_empty() {
            self.canteen_stack.set_visible_child_name("canteen-empty");
        }

        self.canteen_spinner.stop();
        self.canteen_spinner.hide();

        Some(canteen)
    }
}

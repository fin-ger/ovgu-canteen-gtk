use anyhow::{Error, Result};
use gtk::prelude::*;
use gtk::{Box, Builder, Label, Spinner, Stack, ScrolledWindow};
use gettextrs::gettext as t;
use ovgu_canteen::{Canteen, CanteenDescription};

use crate::components::{get, glib_yield, DayComponent, WindowComponent, GLADE};
use crate::util::{enclose, AdjustingVec};

pub struct CanteenComponent {
    description: CanteenDescription,
    canteen_stack: Stack,
    canteen_error_label: Label,
    canteen_spinner: Spinner,
    days: AdjustingVec<DayComponent, Error>,
}

pub fn translate(description: &CanteenDescription) -> String {
    log::debug!("translating canteen {:?}", description);

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
        log::debug!("creating new CanteenComponent for canteen {:?}", description);

        let builder = Builder::new_from_string(GLADE);
        let canteen_stack: Stack = get!(&builder, "canteen-stack")?;
        let canteen_scrolled_window: ScrolledWindow = get!(&builder, "canteen-scrolled-window")?;
        let canteen_error_label: Label = get!(&builder, "canteen-error-label")?;
        let canteen_spinner: Spinner = get!(&builder, "canteen-spinner")?;
        let days_box: Box = get!(&builder, "days-box")?;
        let canteen_name = translate(description);

        log::debug!("adding CanteenComponent {:?} to window", description);
        window.add_canteen(&canteen_stack, serde_plain::to_string(description).unwrap(), canteen_name)?;

        // create a new adjusting vector which adjusts its size according to an input iterator
        let days = AdjustingVec::new(
            // define how to create a new DayComponent
            enclose! { (canteen_scrolled_window, description, days_box) move || {
                enclose! { (canteen_scrolled_window, description, days_box) async move {
                    let comp = DayComponent::new(move |y| {
                        Self::scroll_to(&canteen_scrolled_window, &description, y);
                    }).await?;
                    days_box.pack_start(comp.root_widget(), false, true, 0);

                    glib_yield!(); // give gtk a chance to update the UI
                    Ok(comp)
                }}
            }},
            // define how to delete a DayComponent
            |day| async move {
                day.root_widget().destroy();
                glib_yield!(); // give gtk a chance to update the UI
                Ok(())
            },
        );

        log::debug!("finish creating new CanteenComponent for canteen {:?}", description);

        Ok(Self {
            description: description.clone(),
            canteen_stack,
            canteen_error_label,
            canteen_spinner,
            days,
        })
    }

    fn scroll_to(canteen_scrolled_window: &ScrolledWindow, description: &CanteenDescription, y: i32) {
        if let Some(position) = canteen_scrolled_window.get_vadjustment() {
            log::debug!("scrolling to todays canteen in CanteenComponent {:?}", description);
            position.set_value(y as f64 - 42.0);
        }
    }

    pub async fn load(&mut self, load_result: Result<Canteen>) -> Option<Canteen> {
        log::debug!("loading content into CanteenComponent {:?}", self.description);

        // start and show loading spinner
        self.canteen_spinner.start();
        self.canteen_spinner.show();

        let canteen = match load_result {
            Ok(canteen) => {
                log::debug!("unpacking content for CanteenComponent {:?}", self.description);

                // makes the menu items visible in this canteen-component,
                // e.g. not the error page if previously shown
                self.canteen_stack.set_visible_child_name("canteen-menu");

                canteen
            }
            Err(e) => {
                // makes the error page visible in this canteen-component,
                self.canteen_stack.set_visible_child_name("canteen-error");
                // show error message in UI to inform the user
                self.canteen_error_label
                    .set_text(&format!("{}: {:#}", t("error"), e));
                log::error!("error unpacking content for CanteenComponent {:?}: {:#}", self.description, e);
                return None;
            }
        };

        log::debug!("loading days into CanteenComponent {:?}", self.description);

        // adjust DayComponents to match canteen.days
        let days_result = self
            .days
            .adjust(&canteen.days, |mut comp, day| async move {
                // how to update a DayComponent
                comp.load(day).await;
                glib_yield!(); // give gtk a chance to update the UI
                Ok(comp)
            })
            .await;

        if let Err(e) = days_result {
            // make the error page visible for this canteen-component
            self.canteen_stack.set_visible_child_name("canteen-error");
            // show error message in UI to inform the user
            self.canteen_error_label
                .set_text(&format!("{}: {:#}", t("error"), e));
            log::error!("error loading days into CanteenComponent {:?}: {:#}", self.description, e);
        } else if canteen.days.is_empty() {
            // make the canteen-empty page visible for this canteen-component
            // informs the user that there are no menus available for this canteen
            self.canteen_stack.set_visible_child_name("canteen-empty");
            log::info!("no days available for CanteenComponent {:?}", self.description);
        }

        // stop and hide loading spinner
        self.canteen_spinner.stop();
        self.canteen_spinner.hide();

        log::debug!("finish loading content into CanteenComponent {:?}", self.description);

        Some(canteen)
    }
}

use anyhow::{Error, Result};
use gtk::prelude::*;
use gtk::{Box, Builder, Spinner, Stack};
use ovgu_canteen::{Canteen, CanteenDescription, Error as CanteenError};

use crate::components::{get, glib_yield, DayComponent, WindowComponent, GLADE};
use crate::util::AdjustingVec;

#[derive(Debug)]
pub struct CanteenComponent {
    canteen_stack: Stack,
    canteen_spinner: Spinner,
    days: AdjustingVec<DayComponent, Error>,
}

impl CanteenComponent {
    pub fn new(description: &CanteenDescription, window: &WindowComponent) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let canteen_stack: Stack = get(&builder, "canteen-stack")?;
        let canteen_spinner: Spinner = get(&builder, "canteen-spinner")?;
        let days_box: Box = get(&builder, "days-box")?;
        let canteen_name = description.to_german_str();

        window.add_canteen(&canteen_stack, canteen_name)?;

        let days = AdjustingVec::new(
            move || {
                let inner_days_box = days_box.clone();

                async move {
                    let comp = DayComponent::new().await?;
                    inner_days_box.pack_start(comp.root_widget(), false, true, 0);
                    glib_yield!();
                    Ok(comp)
                }
            },
            |day| async move {
                day.root_widget().destroy();
                glib_yield!();
                Ok(())
            },
        );

        Ok(Self {
            canteen_spinner,
            canteen_stack,
            days,
        })
    }

    pub async fn load(&mut self, load_result: Result<Canteen, CanteenError>) {
        self.canteen_spinner.start();
        self.canteen_spinner.show();

        let canteen = match load_result {
            Ok(canteen) => canteen,
            Err(e) => {
                eprintln!("error: {}", e);
                self.canteen_stack.set_visible_child_name("canteen-error");
                // TODO: display error message
                return;
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

        if days_result.is_err() {
            // TODO: handle error
        }

        self.canteen_spinner.stop();
        self.canteen_spinner.hide();
    }
}

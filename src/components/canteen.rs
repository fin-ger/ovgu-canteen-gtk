use anyhow::{Result, Error};
use gtk::prelude::*;
use gtk::{Stack, Spinner, Builder, Box};
use ovgu_canteen::{Canteen, CanteenDescription, Error as CanteenError};

use crate::components::{GLADE, get, glib_yield, WindowComponent, DayComponent};
use crate::util::AdjustingVec;

#[derive(Debug)]
pub struct CanteenComponent {
    canteen_stack: Stack,
    canteen_spinner: Spinner,
    days: AdjustingVec<DayComponent, Error>,
}

impl CanteenComponent {
    pub fn new(
        description: CanteenDescription,
        window: &WindowComponent,
    ) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let canteen_stack: Stack = get(&builder, "canteen-stack")?;
        let canteen_spinner: Spinner = get(&builder, "canteen-spinner")?;
        let days_box: Box = get(&builder, "days-box")?;
        let canteen_name = description.to_german_str();

        window.add_canteen(&canteen_stack, canteen_name);

        let days = AdjustingVec::new(
            || async {
                let comp = DayComponent::new().await?;
                days_box.pack_start(comp.root_widget(), false, true, 0);
                glib_yield!();
                Ok(comp)
            },
            |day| async {
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

    pub async fn load(&self, load_result: Result<Canteen, CanteenError>) {
        self.canteen_spinner.start();
        self.canteen_spinner.show();

        let mut canteen = match load_result {
            Ok(canteen) => canteen,
            Err(e) => {
                eprintln!("error: {}", e);
                self.canteen_stack.set_visible_child_name("canteen-error");
                // TODO: display error message
                return;
            }
        };

        self.days.adjust(&canteen.days, |mut comp, day| async {
            comp.load(day);
            glib_yield!();
            comp
        });

        self.canteen_spinner.stop();
        self.canteen_spinner.hide();
    }
}

use anyhow::{Context, Result};
use chrono::{Datelike, TimeZone, Utc, Weekday};
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box, Builder, FlowBox, Frame, Label, ListBox, ListBoxRow, Menu, MenuItem,
    Spinner, Stack,
};
use ovgu_canteen::{Canteen, CanteenDescription, Day, Error as CanteenError, Meal};
use std::cell::RefCell;
use std::rc::Rc;

pub const GLADE: &str = std::include_str!("../data/gnome-ovgu-canteen.glade");

macro_rules! glib_yield {
    () => {
        glib::timeout_future_with_priority(glib::PRIORITY_DEFAULT_IDLE, 0).await
    };
}

#[derive(Debug)]
pub struct CanteenComponent {
    pub canteen_stack: Stack,
    pub canteen_spinner: Spinner,
    pub days_box: Box,
    pub description: CanteenDescription,
}

#[derive(Debug)]
pub struct DayComponent {
    pub frame: Frame,
    pub label: Label,
    pub meals_list_box: ListBox,
}

#[derive(Debug)]
pub struct MealComponent {
    pub meal: ListBoxRow,
    pub name: Label,
    pub badges: FlowBox,
    pub price_student: Label,
    pub price_staff: Label,
    pub price_guest: Label,
}

#[derive(Debug)]
pub struct BadgeComponent {
    pub label: Label,
}

#[derive(Debug)]
pub struct LiteBadgeComponent {
    pub label: Label,
}

#[derive(Debug)]
pub struct WindowComponent {
    pub window: ApplicationWindow,
    pub canteens_stack: Rc<RefCell<Stack>>,
    pub canteens_menu: Menu,
    pub canteen_label: Rc<RefCell<Label>>,
}

impl CanteenComponent {
    pub fn new(description: CanteenDescription, window: &WindowComponent) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let canteen_stack: Stack = builder
            .get_object("canteen-stack")
            .context("'canteen-stack' not available in glade file")?;
        let canteen_spinner: Spinner = builder
            .get_object("canteen-spinner")
            .context("'canteen-spinner' not available in glade file")?;
        let days_box: Box = builder
            .get_object("days-box")
            .context("'days-box' not avaiable in glade file")?;
        let canteen_name = format!("{:?}", description);

        let menu_item = MenuItem::new_with_label(&canteen_name);
        window.canteens_menu.append(&menu_item);
        menu_item.show();
        window
            .canteens_stack
            .borrow_mut()
            .add_named(&canteen_stack, &canteen_name);

        let canteens_stack_handle = Rc::clone(&window.canteens_stack);
        let canteen_label_handle = Rc::clone(&window.canteen_label);
        menu_item.connect_activate(move |_menu_item| {
            canteens_stack_handle
                .borrow()
                .set_visible_child_name(&canteen_name);
            canteen_label_handle.borrow().set_text(&canteen_name);
        });

        Ok(Self {
            canteen_stack,
            canteen_spinner,
            days_box,
            description,
        })
    }

    pub async fn loaded(&self, load_result: Result<Canteen, CanteenError>) {
        match load_result {
            Ok(mut canteen) => {
                for day in canteen.days.drain(..) {
                    match DayComponent::new(&day).await {
                        Ok(day_comp) => {
                            self.days_box.pack_start(&day_comp.frame, false, true, 0);
                        }
                        Err(e) => {
                            eprintln!("error: {}", e);
                            // TODO: add error handling for failed daycomponent
                        }
                    }

                    glib_yield!();
                }
            }
            Err(e) => {
                eprintln!("error: {}", e);
                self.canteen_stack.set_visible_child_name("canteen-error");
                // TODO: display error message
            }
        }

        self.canteen_spinner.stop();
        self.canteen_spinner.hide();
    }
}

impl DayComponent {
    pub async fn new(day: &Day) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let frame: Frame = builder
            .get_object("day-frame")
            .context("'day-frame' not available in glade file")?;
        let label: Label = builder
            .get_object("day-label")
            .context("'day-label' not available in glade file")?;
        let meals_list_box: ListBox = builder
            .get_object("day-meals-list-box")
            .context("'day-meals-list-box' not available in glade file")?;
        let side_dish_badges: FlowBox = builder
            .get_object("side-dish-badges")
            .context("'side-dish-badges' not available in glade file")?;

        let mut day_name = match day.date.weekday() {
            Weekday::Mon => "Montag",
            Weekday::Tue => "Dienstag",
            Weekday::Wed => "Mittwoch",
            Weekday::Thu => "Donnerstag",
            Weekday::Fri => "Freitag",
            Weekday::Sat => "Samstag",
            Weekday::Sun => "Sonntag",
        };
        let today = Utc::today();
        let date = chrono_tz::Europe::Berlin.ymd(day.date.year(), day.date.month(), day.date.day());
        if date == today {
            day_name = "Heute";
        }
        if date == today.succ() {
            day_name = "Morgen";
        }

        label.set_text(day_name);

        for (idx, meal) in day.meals.iter().enumerate() {
            match MealComponent::new(meal).await {
                Ok(meal_component) => {
                    meals_list_box.insert(&meal_component.meal, idx as i32);
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    // TODO: handle meal creation failure
                }
            }

            glib_yield!();
        }

        for side_dish in &day.side_dishes {
            let badge = BadgeComponent::new(side_dish)
                .await
                .context("side dishes could not be created")?;
            side_dish_badges.insert(&badge.label, 0);
            glib_yield!();
        }

        if day.side_dishes.is_empty() {
            let badge = LiteBadgeComponent::new("nicht vorhanden")
                .await
                .context("empty side dishes note could not be created")?;
            side_dish_badges.insert(&badge.label, 0);
            glib_yield!();
        }

        Ok(Self {
            frame,
            label,
            meals_list_box,
        })
    }
}

impl MealComponent {
    pub async fn new(meal: &Meal) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let meal_box: ListBoxRow = builder
            .get_object("meal")
            .context("'meal' not available in glade file")?;
        let name: Label = builder
            .get_object("meal-name")
            .context("'meal-name' not available in glade file")?;
        let badges: FlowBox = builder
            .get_object("badges")
            .context("'badges' not available in glade file")?;
        let price_student: Label = builder
            .get_object("meal-price-student")
            .context("'meal-price-student' not available in glade file")?;
        let price_staff: Label = builder
            .get_object("meal-price-staff")
            .context("'meal-price-staff' not available in glade file")?;
        let price_guest: Label = builder
            .get_object("meal-price-guest")
            .context("'meal-price-guest' not available in glade file")?;

        name.set_text(&meal.name);
        price_student.set_text(format!("{:.2} €", meal.price.student).as_str());
        price_staff.set_text(format!("{:.2} €", meal.price.staff).as_str());
        price_guest.set_text(format!("{:.2} €", meal.price.guest).as_str());

        for additive in &meal.additives {
            let badge = LiteBadgeComponent::new(additive.to_german_str()).await?;
            badges.insert(&badge.label, 0);
            glib_yield!();
        }

        for allergenic in &meal.allergenics {
            let badge = LiteBadgeComponent::new(allergenic.to_german_str()).await?;
            badges.insert(&badge.label, 0);
            glib_yield!();
        }

        for symbol in &meal.symbols {
            let badge = BadgeComponent::new(symbol.to_german_str()).await?;
            badges.insert(&badge.label, 0);
            glib_yield!();
        }

        Ok(Self {
            meal: meal_box,
            name,
            badges,
            price_student,
            price_staff,
            price_guest,
        })
    }
}

impl BadgeComponent {
    pub async fn new(text: &str) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let label: Label = builder
            .get_object("badge")
            .context("'badge' not available in glade file")?;

        label.set_text(text);

        Ok(Self { label })
    }
}

impl LiteBadgeComponent {
    pub async fn new(text: &str) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let label: Label = builder
            .get_object("lite-badge")
            .context("'lite-badge' not available in glade file")?;

        label.set_text(text);

        Ok(Self { label })
    }
}

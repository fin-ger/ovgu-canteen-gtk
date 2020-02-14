use gtk::{
    Builder,
    Frame,
    Label,
    ListBox,
    ListBoxRow,
    FlowBox,
    Box,
    Stack,
    MenuItem,
    ApplicationWindow,
};
use gtk::prelude::*;
use ovgu_canteen::{Day, Meal};
use chrono::{Datelike, Weekday, Utc, TimeZone};

use std::rc::Rc;
use std::cell::RefCell;

pub const GLADE: &str = std::include_str!("../data/gnome-ovgu-canteen.glade");

#[macro_export]
macro_rules! glib_yield {
    () => {
        glib::timeout_future_with_priority(glib::PRIORITY_DEFAULT_IDLE, 0).await
    }
}

#[derive(Debug)]
pub struct DayComponent {
    pub frame: Frame,
    pub label: Label,
    pub meals_list_box: ListBox,
}

pub struct MealComponent {
    pub meal: ListBoxRow,
    pub name: Label,
    pub badges: FlowBox,
    pub price_student: Label,
    pub price_staff: Label,
    pub price_guest: Label,
}

pub struct BadgeComponent {
    pub label: Label,
}

pub struct LiteBadgeComponent {
    pub label: Label,
}

pub struct WindowComponent {
    pub window: Rc<RefCell<ApplicationWindow>>,
    pub canteen_stack: Rc<RefCell<Stack>>,
    pub canteen_label: Rc<RefCell<Label>>,
    pub lower_hall_days_box: Rc<RefCell<Box>>,
    pub upper_hall_days_box: Rc<RefCell<Box>>,
    pub kellercafe_days_box: Rc<RefCell<Box>>,
    pub herrenkrug_days_box: Rc<RefCell<Box>>,
    pub stendal_days_box: Rc<RefCell<Box>>,
    pub wernigerode_days_box: Rc<RefCell<Box>>,
    pub dom_cafete_days_box: Rc<RefCell<Box>>,
    pub lower_hall_item: Rc<RefCell<MenuItem>>,
    pub upper_hall_item: Rc<RefCell<MenuItem>>,
    pub kellercafe_item: Rc<RefCell<MenuItem>>,
    pub herrenkrug_item: Rc<RefCell<MenuItem>>,
    pub stendal_item: Rc<RefCell<MenuItem>>,
    pub wernigerode_item: Rc<RefCell<MenuItem>>,
    pub dom_cafete_item: Rc<RefCell<MenuItem>>,
}

pub struct WindowComponentBuilder {
    pub window: ApplicationWindow,
    pub canteen_stack: Stack,
    pub canteen_label: Label,
    pub lower_hall_days_box: Box,
    pub upper_hall_days_box: Box,
    pub kellercafe_days_box: Box,
    pub herrenkrug_days_box: Box,
    pub stendal_days_box: Box,
    pub wernigerode_days_box: Box,
    pub dom_cafete_days_box: Box,
    pub lower_hall_item: MenuItem,
    pub upper_hall_item: MenuItem,
    pub kellercafe_item: MenuItem,
    pub herrenkrug_item: MenuItem,
    pub stendal_item: MenuItem,
    pub wernigerode_item: MenuItem,
    pub dom_cafete_item: MenuItem,
}

impl WindowComponentBuilder {
    pub fn build(self) -> WindowComponent {
        WindowComponent {
            window: Rc::new(RefCell::new(self.window)),
            canteen_stack: Rc::new(RefCell::new(self.canteen_stack)),
            canteen_label: Rc::new(RefCell::new(self.canteen_label)),
            lower_hall_days_box: Rc::new(RefCell::new(self.lower_hall_days_box)),
            upper_hall_days_box: Rc::new(RefCell::new(self.upper_hall_days_box)),
            kellercafe_days_box: Rc::new(RefCell::new(self.kellercafe_days_box)),
            herrenkrug_days_box: Rc::new(RefCell::new(self.herrenkrug_days_box)),
            stendal_days_box: Rc::new(RefCell::new(self.stendal_days_box)),
            wernigerode_days_box: Rc::new(RefCell::new(self.wernigerode_days_box)),
            dom_cafete_days_box: Rc::new(RefCell::new(self.dom_cafete_days_box)),
            lower_hall_item: Rc::new(RefCell::new(self.lower_hall_item)),
            upper_hall_item: Rc::new(RefCell::new(self.upper_hall_item)),
            kellercafe_item: Rc::new(RefCell::new(self.kellercafe_item)),
            herrenkrug_item: Rc::new(RefCell::new(self.herrenkrug_item)),
            stendal_item: Rc::new(RefCell::new(self.stendal_item)),
            wernigerode_item: Rc::new(RefCell::new(self.wernigerode_item)),
            dom_cafete_item: Rc::new(RefCell::new(self.dom_cafete_item)),
        }
    }
}

impl DayComponent {
    pub async fn new(day: &Day) -> DayComponent {
        let builder = Builder::new_from_string(GLADE);
        let frame: Frame = builder.get_object("day-frame").unwrap();
        let label: Label = builder.get_object("day-label").unwrap();
        let meals_list_box: ListBox = builder.get_object("day-meals-list-box").unwrap();
        let side_dish_badges: FlowBox = builder.get_object("side-dish-badges").unwrap();

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
        let date = chrono_tz::Europe::Berlin.ymd(
            day.date.year(),
            day.date.month(),
            day.date.day(),
        );
        if date == today {
            day_name = "Heute";
        }
        if date == today.succ() {
            day_name = "Morgen";
        }

        label.set_text(day_name);

        for (idx, meal) in day.meals.iter().enumerate() {
            let meal_component = MealComponent::new(meal).await;
            meals_list_box.insert(&meal_component.meal, idx as i32);
            glib_yield!();
        }

        for side_dish in day.side_dishes.iter() {
            let badge = BadgeComponent::new(side_dish).await;
            side_dish_badges.insert(&badge.label, 0);
            glib_yield!();
        }

        if day.side_dishes.len() == 0 {
            let badge = LiteBadgeComponent::new("nicht vorhanden").await;
            side_dish_badges.insert(&badge.label, 0);
            glib_yield!();
        }

        DayComponent {
            frame,
            label,
            meals_list_box,
        }
    }
}

impl MealComponent {
    pub async fn new(meal: &Meal) -> MealComponent {
        let builder = Builder::new_from_string(GLADE);
        let meal_box: ListBoxRow = builder.get_object("meal").unwrap();
        let name: Label = builder.get_object("meal-name").unwrap();
        let badges: FlowBox = builder.get_object("badges").unwrap();
        let price_student: Label = builder.get_object("meal-price-student").unwrap();
        let price_staff: Label = builder.get_object("meal-price-staff").unwrap();
        let price_guest: Label = builder.get_object("meal-price-guest").unwrap();

        name.set_text(&meal.name);
        price_student.set_text(format!("{:.2} €", meal.price.student).as_str());
        price_staff.set_text(format!("{:.2} €", meal.price.staff).as_str());
        price_guest.set_text(format!("{:.2} €", meal.price.guest).as_str());

        for additive in meal.additives.iter() {
            let badge = LiteBadgeComponent::new(additive.to_german_str()).await;
            badges.insert(&badge.label, 0);
            glib_yield!();
        }

        for allergenic in meal.allergenics.iter() {
            let badge = LiteBadgeComponent::new(allergenic.to_german_str()).await;
            badges.insert(&badge.label, 0);
            glib_yield!();
        }

        for symbol in meal.symbols.iter() {
            let badge = BadgeComponent::new(symbol.to_german_str()).await;
            badges.insert(&badge.label, 0);
            glib_yield!();
        }

        MealComponent {
            meal: meal_box,
            name,
            badges,
            price_student,
            price_staff,
            price_guest,
        }
    }
}

impl BadgeComponent {
    pub async fn new(text: &str) -> BadgeComponent {
        let builder = Builder::new_from_string(GLADE);
        let label: Label = builder.get_object("badge").unwrap();

        label.set_text(text);

        BadgeComponent {
            label
        }
    }
}

impl LiteBadgeComponent {
    pub async fn new(text: &str) -> LiteBadgeComponent {
        let builder = Builder::new_from_string(GLADE);
        let label: Label = builder.get_object("lite-badge").unwrap();

        label.set_text(text);

        LiteBadgeComponent {
            label
        }
    }
}

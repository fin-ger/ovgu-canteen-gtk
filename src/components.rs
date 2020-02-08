use gtk::{
    Builder,
    AboutDialog,
    Frame,
    Label,
    ListBox,
    ListBoxRow,
    FlowBox,
    Box,
    ApplicationWindow,
};
use gtk::prelude::*;
use ovgu_canteen::{Day, Meal};
use chrono::Datelike;

pub const GLADE: &str = std::include_str!("../data/gnome-ovgu-canteen.glade");

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
    pub window: ApplicationWindow,
    pub lower_hall_days_box: Box,
    pub upper_hall_days_box: Box,
}

pub struct AboutComponent {
    pub dialog: AboutDialog,
}

impl DayComponent {
    pub fn new(day: &Day) -> DayComponent {
        let builder = Builder::new_from_string(GLADE);
        let frame: Frame = builder.get_object("day-frame").unwrap();
        let label: Label = builder.get_object("day-label").unwrap();
        let meals_list_box: ListBox = builder.get_object("day-meals-list-box").unwrap();

        // TODO: display Heute, Morgen, <Wochentage>
        //       implement this in rust-ovgu-canteen::Day
        label.set_text(&day.date.weekday().to_string());

        for (idx, meal) in day.meals.iter().enumerate() {
            let meal_component = MealComponent::new(meal);
            meals_list_box.insert(&meal_component.meal, idx as i32);
        }

        // TODO: add side-dishes

        DayComponent {
            frame,
            label,
            meals_list_box,
        }
    }
}

impl MealComponent {
    pub fn new(meal: &Meal) -> MealComponent {
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
            // TODO: convert additives to strings
            //       do this in rust-ovgu-canteen::Additive
            let badge = LiteBadgeComponent::new(format!("{:?}", additive));
            badges.insert(&badge.label, 0);
        }

        for allergenic in meal.allergenics.iter() {
            // TODO: convert allergenics to strings
            //       do this in rust-ovgu-canteen::Allergenic
            let badge = LiteBadgeComponent::new(format!("{:?}", allergenic));
            badges.insert(&badge.label, 0);
        }

        for symbol in meal.symbols.iter() {
            // TODO: convert symbols to strings
            //       do this in rust-ovgu-canteen::Symbol
            let badge = BadgeComponent::new(format!("{:?}", symbol));
            badges.insert(&badge.label, 0);
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
    pub fn new(text: String) -> BadgeComponent {
        let builder = Builder::new_from_string(GLADE);
        let label: Label = builder.get_object("badge").unwrap();

        label.set_text(&text);

        BadgeComponent {
            label
        }
    }
}

impl LiteBadgeComponent {
    pub fn new(text: String) -> LiteBadgeComponent {
        let builder = Builder::new_from_string(GLADE);
        let label: Label = builder.get_object("lite-badge").unwrap();

        label.set_text(&text);

        LiteBadgeComponent {
            label
        }
    }
}

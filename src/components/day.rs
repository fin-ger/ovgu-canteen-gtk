use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicI32};

use anyhow::{Error, Result};
use chrono::{Datelike, TimeZone, Utc, Weekday};
use gtk::prelude::*;
use gtk::{Builder, FlowBox, Frame, Label, ListBox};
use ovgu_canteen::Day;

use crate::components::{
    get, glib_yield, BadgeComponent, LiteBadgeComponent, MealComponent, GLADE,
};
use crate::util::{enclose, AdjustingVec};

#[derive(Debug)]
pub struct DayComponent {
    frame: Frame,
    label: Label,
    side_dish_badges: FlowBox,
    empty_side_dishes_label: Option<Label>,
    meals: AdjustingVec<MealComponent, Error>,
    side_dishes: AdjustingVec<BadgeComponent, Error>,
}

impl DayComponent {
    pub async fn new() -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let frame: Frame = get(&builder, "day-frame")?;
        let label: Label = get(&builder, "day-label")?;
        let meals_list_box: ListBox = get(&builder, "day-meals-list-box")?;
        let side_dish_badges: FlowBox = get(&builder, "side-dish-badges")?;

        let meal_offset = Arc::new(AtomicI32::new(0));
        let side_dish_offset = Arc::new(AtomicI32::new(0));

        let meals = AdjustingVec::new(
            enclose! { (meals_list_box, meal_offset) move || {
                enclose! { (meals_list_box, meal_offset) async move {
                    let comp = MealComponent::new().await?;
                    meals_list_box.insert(comp.root_widget(), meal_offset.load(Ordering::SeqCst));
                    meal_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (meal_offset) move |meal| {
                enclose! { (meal_offset) async move {
                    meal.root_widget().destroy();
                    meal_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }}
            }},
        );

        let side_dishes = AdjustingVec::new(
            enclose! { (side_dish_badges, side_dish_offset) move || {
                enclose! { (side_dish_badges, side_dish_offset) async move {
                    let comp = BadgeComponent::new().await?;
                    side_dish_badges.insert(comp.root_widget(), side_dish_offset.load(Ordering::SeqCst));
                    side_dish_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (side_dish_offset) move |badge| {
                enclose! { (side_dish_offset) async move {
                    badge.root_widget().destroy();
                    side_dish_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }}
            }},
        );

        Ok(Self {
            empty_side_dishes_label: None,
            side_dish_badges,
            frame,
            label,
            meals,
            side_dishes,
        })
    }

    pub const fn root_widget(&self) -> &Frame {
        &self.frame
    }

    pub async fn load(&mut self, day: &Day) {
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

        self.label.set_text(day_name);

        let meal_result = self
            .meals
            .adjust(&day.meals, |mut comp, meal| async move {
                comp.load(meal).await?;
                glib_yield!();
                Ok(comp)
            })
            .await;

        if meal_result.is_err() {
            // TODO: handle error
        }

        let side_dish_result = self
            .side_dishes
            .adjust(&day.side_dishes, |badge, side_dish| async move {
                badge.load(side_dish).await;
                glib_yield!();
                Ok(badge)
            })
            .await;

        if side_dish_result.is_err() {
            // TODO: handle error
        }

        if day.side_dishes.is_empty() && self.empty_side_dishes_label.is_none() {
            let badge = match LiteBadgeComponent::new().await {
                Ok(badge) => badge,
                Err(_e) => {
                    // TODO: handle error
                    unimplemented!();
                }
            };
            badge.load("nicht vorhanden").await;
            self.side_dish_badges.insert(badge.root_widget(), 0);
            glib_yield!();
        } else if !day.side_dishes.is_empty() && self.empty_side_dishes_label.is_some() {
            self.empty_side_dishes_label.take().unwrap().destroy();
        }
    }
}

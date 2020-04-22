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
use crate::util::AdjustingVec;

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

        let meal_offset_create = Arc::new(AtomicI32::new(0));
        let meal_offset_destroy = Arc::clone(&meal_offset_create);
        let side_dish_offset_create = Arc::new(AtomicI32::new(0));
        let side_dish_offset_destroy = Arc::clone(&side_dish_offset_create);

        let meals = AdjustingVec::new(
            move || {
                let inner_meals_list_box = meals_list_box.clone();
                let inner_meal_offset = Arc::clone(&meal_offset_create);

                async move {
                    let comp = MealComponent::new().await?;
                    inner_meals_list_box.insert(comp.root_widget(), inner_meal_offset.load(Ordering::SeqCst));

                    inner_meal_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }
            },
            move |meal| {
                let inner_meal_offset = Arc::clone(&meal_offset_destroy);

                async move {
                    meal.root_widget().destroy();

                    inner_meal_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }
            },
        );

        let badges = side_dish_badges.clone();
        let side_dishes = AdjustingVec::new(
            move || {
                let inner_side_dish_badges = badges.clone();
                let inner_side_dish_offset = Arc::clone(&side_dish_offset_create);

                async move {
                    let comp = BadgeComponent::new().await?;
                    inner_side_dish_badges.insert(comp.root_widget(), inner_side_dish_offset.load(Ordering::SeqCst));

                    inner_side_dish_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }
            },
            move |badge| {
                let inner_side_dish_offset = Arc::clone(&side_dish_offset_destroy);

                async move {
                    badge.root_widget().destroy();

                    inner_side_dish_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }
            },
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

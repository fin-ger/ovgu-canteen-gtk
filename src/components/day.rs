use std::sync::atomic::{AtomicI32, AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Error, Result};
use chrono::{Datelike, TimeZone, Utc, Weekday};
use gtk::prelude::*;
use gtk::{Builder, FlowBox, Frame, InfoBar, Label, ListBox};
use gettextrs::gettext as t;
use ovgu_canteen::Day;

use crate::components::{
    get, glib_yield, BadgeComponent, LiteBadgeComponent, MealComponent, GLADE,
};
use crate::util::{enclose, AdjustingVec};

#[derive(Debug)]
pub struct DayComponent {
    frame: Frame,
    label: Label,
    date_label: Label,
    error: InfoBar,
    error_label: Label,
    side_dish_badges: FlowBox,
    empty_side_dishes_label: Option<LiteBadgeComponent>,
    meals: AdjustingVec<MealComponent, Error>,
    side_dishes: AdjustingVec<BadgeComponent, Error>,
    is_today: Arc<AtomicBool>,
}

impl DayComponent {
    pub async fn new<F: Fn(i32) + 'static>(scroll_to: F) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let frame: Frame = get!(&builder, "day-frame")?;
        let label: Label = get!(&builder, "day-label")?;
        let date_label: Label = get!(&builder, "date-label")?;
        let error: InfoBar = get!(&builder, "day-error")?;
        let error_label: Label = get!(&builder, "day-error-label")?;
        let meals_list_box: ListBox = get!(&builder, "day-meals-list-box")?;
        let side_dish_badges: FlowBox = get!(&builder, "side-dish-badges")?;

        let is_today = Arc::new(AtomicBool::new(false));

        frame.connect_size_allocate(enclose! { (is_today) move |_frame, allocation| {
            if is_today.load(Ordering::SeqCst) {
                scroll_to(allocation.y);
            }
        }});

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
                    // a flowbox item always has a parent - a FlowBoxChild
                    badge.root_widget().get_parent().unwrap().destroy();
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
            date_label,
            error,
            error_label,
            meals,
            side_dishes,
            is_today,
        })
    }

    pub const fn root_widget(&self) -> &Frame {
        &self.frame
    }

    pub async fn load(&mut self, day: &Day) {
        let mut day_name = match day.date.weekday() {
            Weekday::Mon => t("Monday"),
            Weekday::Tue => t("Tuesday"),
            Weekday::Wed => t("Wednesday"),
            Weekday::Thu => t("Thursday"),
            Weekday::Fri => t("Friday"),
            Weekday::Sat => t("Saturday"),
            Weekday::Sun => t("Sunday"),
        };
        let today = Utc::today();
        let date = chrono_tz::Europe::Berlin.ymd(day.date.year(), day.date.month(), day.date.day());
        if date == today {
            day_name = t("Today");
            self.is_today.store(true, Ordering::SeqCst);
        } else {
            self.is_today.store(false, Ordering::SeqCst);
        }
        if date == today.succ() {
            day_name = t("Tomorrow");
        }

        self.label.set_text(&day_name);
        self.date_label.set_text(&format!("{}", day.date.format("%d.%m.%Y")));

        let meal_result = self
            .meals
            .adjust(&day.meals, |mut comp, meal| async move {
                comp.load(meal).await?;
                glib_yield!();
                Ok(comp)
            })
            .await;

        let side_dish_result = self
            .side_dishes
            .adjust(&day.side_dishes, |badge, side_dish| async move {
                badge.load(side_dish).await;
                glib_yield!();
                Ok(badge)
            })
            .await;

        if day.side_dishes.is_empty() && self.empty_side_dishes_label.is_none() {
            // this cannot fail as the badge component always returns Ok
            let badge = LiteBadgeComponent::new().await.unwrap();
            badge.load(&t("not available")).await;
            self.side_dish_badges.insert(badge.root_widget(), 0);
            self.empty_side_dishes_label = Some(badge);

            glib_yield!();
        } else if !day.side_dishes.is_empty() && self.empty_side_dishes_label.is_some() {
            self.empty_side_dishes_label
                .take()
                .unwrap() // checked above
                .root_widget()
                .get_parent()
                .unwrap() // a flowbox item always has a parent - a FlowBoxChild
                .destroy();
        }

        let mut error_msg = None;
        if let Err(e) = meal_result {
            error_msg = Some(format!("{}: {:#}", t("error"), e));
        }

        if let Err(e) = side_dish_result {
            let msg = format!("{}: {:#}", t("error"), e);
            error_msg = if let Some(prev_msg) = error_msg {
                Some(format!("{}\n{}", prev_msg, msg))
            } else {
                Some(msg)
            };
        }

        if let Some(msg) = error_msg {
            self.error.show_all();
            self.error_label.set_text(&msg);
        } else {
            self.error.hide();
        }
    }
}

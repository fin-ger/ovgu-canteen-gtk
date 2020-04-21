use anyhow::{Result, Error};
use gtk::prelude::*;
use gtk::{Label, ListBoxRow, Builder, FlowBox};
use ovgu_canteen::Meal;

use crate::components::{GLADE, get, glib_yield, BadgeComponent, LiteBadgeComponent};
use crate::util::AdjustingVec;

#[derive(Debug)]
pub struct MealComponent {
    name: Label,
    meal: ListBoxRow,
    price_student: Label,
    price_staff: Label,
    price_guest: Label,
    additives: AdjustingVec<LiteBadgeComponent, Error>,
    allergenics: AdjustingVec<LiteBadgeComponent, Error>,
    symbols: AdjustingVec<BadgeComponent, Error>,
}

impl MealComponent {
    pub async fn new() -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let meal_box: ListBoxRow = get(&builder, "meal")?;
        let name: Label = get(&builder, "meal-name")?;
        let badges: FlowBox = get(&builder, "badges")?;
        let price_student: Label = get(&builder, "meal-price-student")?;
        let price_staff: Label = get(&builder, "meal-price-staff")?;
        let price_guest: Label = get(&builder, "meal-price-guest")?;

        let mut additive_offset = 0;
        let mut allergenic_offset = 0;
        let mut symbol_offset = 0;

        let additives = AdjustingVec::new(
            || async {
                let comp = LiteBadgeComponent::new().await?;
                badges.insert(comp.root_widget(), additive_offset);

                additive_offset += 1;
                allergenic_offset += 1;
                symbol_offset += 1;

                glib_yield!();
                Ok(comp)
            },
            |badge| async {
                badge.root_widget().destroy();

                additive_offset -= 1;
                allergenic_offset -= 1;
                symbol_offset -= 1;

                glib_yield!();
                Ok(())
            },
        );

        let allergenics = AdjustingVec::new(
            || async {
                let comp = LiteBadgeComponent::new().await?;
                badges.insert(comp.root_widget(), allergenic_offset);

                allergenic_offset += 1;
                symbol_offset += 1;

                glib_yield!();
                Ok(comp)
            },
            |badge| async {
                badge.root_widget().destroy();

                allergenic_offset -= 1;
                symbol_offset -= 1;

                glib_yield!();
                Ok(())
            },
        );
        let symbols = AdjustingVec::new(
            || async {
                let comp = BadgeComponent::new().await?;
                badges.insert(comp.root_widget(), symbol_offset);

                symbol_offset += 1;

                glib_yield!();
                Ok(comp)
            },
            |badge| async {
                badge.root_widget().destroy();

                symbol_offset -= 1;

                glib_yield!();
                Ok(())
            },
        );

        Ok(Self {
            meal: meal_box,
            name,
            price_student,
            price_staff,
            price_guest,
            additives,
            allergenics,
            symbols,
        })
    }

    pub fn root_widget(&self) -> &ListBoxRow {
        &self.meal
    }

    pub async fn load(&mut self, meal: &Meal) {
        self.name.set_text(&meal.name);
        self.price_student.set_text(format!("{:.2} €", meal.price.student).as_str());
        self.price_staff.set_text(format!("{:.2} €", meal.price.staff).as_str());
        self.price_guest.set_text(format!("{:.2} €", meal.price.guest).as_str());

        self.additives.adjust(
            &meal.additives,
            |badge, additive| async {
                badge.load(additive.to_german_str());
                glib_yield!();
                badge
            },
        ).await;

        self.allergenics.adjust(
            &meal.allergenics,
            |badge, allergenic| async {
                badge.load(allergenic.to_german_str());
                glib_yield!();
                badge
            },
        ).await;

        self.symbols.adjust(
            &meal.symbols,
            |badge, symbol| async {
                badge.load(symbol.to_german_str());
                glib_yield!();
                badge
            },
        ).await;
    }
}

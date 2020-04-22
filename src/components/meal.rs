use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use anyhow::{Error, Result};
use gtk::prelude::*;
use gtk::{Builder, FlowBox, Label, ListBoxRow};
use ovgu_canteen::Meal;

use crate::components::{get, glib_yield, BadgeComponent, LiteBadgeComponent, GLADE};
use crate::util::{enclose, AdjustingVec};

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

        let additive_offset = Arc::new(AtomicI32::new(0));
        let allergenic_offset = Arc::new(AtomicI32::new(0));
        let symbol_offset = Arc::new(AtomicI32::new(0));

        let additives = AdjustingVec::new(
            enclose! { (badges, additive_offset, allergenic_offset, symbol_offset) move || {
                enclose! { (badges, additive_offset, allergenic_offset, symbol_offset) async move {
                    let comp = LiteBadgeComponent::new().await?;
                    badges.insert(comp.root_widget(), additive_offset.load(Ordering::SeqCst));
                    additive_offset.fetch_add(1, Ordering::SeqCst);
                    allergenic_offset.fetch_add(1, Ordering::SeqCst);
                    symbol_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (additive_offset, allergenic_offset, symbol_offset) move |badge| {
                enclose! { (additive_offset, allergenic_offset, symbol_offset) async move {
                    badge.root_widget().destroy();
                    additive_offset.fetch_sub(1, Ordering::SeqCst);
                    allergenic_offset.fetch_sub(1, Ordering::SeqCst);
                    symbol_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }}
            }},
        );

        let allergenics = AdjustingVec::new(
            enclose! { (badges, allergenic_offset, symbol_offset) move || {
                enclose! { (badges, allergenic_offset, symbol_offset) async move {
                    let comp = LiteBadgeComponent::new().await?;
                    badges.insert(comp.root_widget(), allergenic_offset.load(Ordering::SeqCst));
                    allergenic_offset.fetch_add(1, Ordering::SeqCst);
                    symbol_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (allergenic_offset, symbol_offset) move |badge| {
                enclose! { (allergenic_offset, symbol_offset) async move {
                    badge.root_widget().destroy();
                    allergenic_offset.fetch_sub(1, Ordering::SeqCst);
                    symbol_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }}
            }},
        );

        let symbols = AdjustingVec::new(
            enclose! { (badges, symbol_offset) move || {
                enclose! { (badges, symbol_offset) async move {
                    let comp = BadgeComponent::new().await?;
                    badges.insert(comp.root_widget(), symbol_offset.load(Ordering::SeqCst));
                    symbol_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (symbol_offset) move |badge| {
                enclose! { (symbol_offset) async move {
                    badge.root_widget().destroy();
                    symbol_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }}
            }},
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

    pub const fn root_widget(&self) -> &ListBoxRow {
        &self.meal
    }

    pub async fn load(&mut self, meal: &Meal) -> Result<()> {
        self.name.set_text(&meal.name);
        self.price_student
            .set_text(format!("{:.2} €", meal.price.student).as_str());
        self.price_staff
            .set_text(format!("{:.2} €", meal.price.staff).as_str());
        self.price_guest
            .set_text(format!("{:.2} €", meal.price.guest).as_str());

        self.additives
            .adjust(&meal.additives, |badge, additive| async move {
                badge.load(additive.to_german_str()).await;
                glib_yield!();
                Ok(badge)
            })
            .await?;

        self.allergenics
            .adjust(&meal.allergenics, |badge, allergenic| async move {
                badge.load(allergenic.to_german_str()).await;
                glib_yield!();
                Ok(badge)
            })
            .await?;

        self.symbols
            .adjust(&meal.symbols, |badge, symbol| async move {
                badge.load(symbol.to_german_str()).await;
                glib_yield!();
                Ok(badge)
            })
            .await?;

        Ok(())
    }
}

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use anyhow::{Error, Result};
use gtk::prelude::*;
use gtk::{Builder, FlowBox, Label, ListBoxRow};
use gettextrs::gettext as t;
use ovgu_canteen::{Meal, Additive, Allergenic, Symbol};

use crate::components::{get, glib_yield, SymbolComponent, LiteBadgeComponent, GLADE};
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
    symbols: AdjustingVec<SymbolComponent, Error>,
}

fn translate_additive(additive: &Additive) -> String {
    match additive {
        Additive::FoodColoring => t("Food Coloring"),
        Additive::FoodPreservatives => t("Food Preservatives"),
        Additive::AntiOxidants => t("Anti Oxidants"),
        Additive::FlavorEnhancer => t("Flavor Enhancer"),
        Additive::Sulfurized => t("Sulfurized"),
        Additive::Waxed => t("Waxed"),
        Additive::Blackend => t("Blackend"),
        Additive::Phosphates => t("Phosphates"),
        Additive::Sweetener => t("Sweetener"),
        Additive::Phenylalanine => t("Phenylalanine"),
    }
}

fn translate_allergenic(allergenic: &Allergenic) -> String {
    match allergenic {
        Allergenic::Wheat => t("Wheat"),
        Allergenic::Rye => t("Rye"),
        Allergenic::Barley => t("Barley"),
        Allergenic::Oat => t("Oat"),
        Allergenic::Spelt => t("Spelt"),
        Allergenic::Kamut => t("Kamut"),
        Allergenic::Crustacean => t("Crustacean"),
        Allergenic::Egg => t("Egg"),
        Allergenic::Fish => t("Fish"),
        Allergenic::Peanut => t("Peanut"),
        Allergenic::Soya => t("Soya"),
        Allergenic::Lactose => t("Lactose"),
        Allergenic::Almond => t("Almond"),
        Allergenic::Hazelnut => t("Hazelnut"),
        Allergenic::Walnut => t("Walnut"),
        Allergenic::Cashew => t("Cashew"),
        Allergenic::PecanNut => t("Pecan Nut"),
        Allergenic::BrazilNut => t("Brazil Nut"),
        Allergenic::Pistachio => t("Pistachio"),
        Allergenic::MacadamiaNut => t("Macadamia Nut"),
        Allergenic::QueenslandNut => t("Queensland Nut"),
        Allergenic::Celery => t("Celery"),
        Allergenic::Mustard => t("Mustard"),
        Allergenic::Sesame => t("Sesame"),
        Allergenic::Sulphite => t("Sulphite"),
        Allergenic::Lupin => t("Lupin"),
        Allergenic::Mollusc => t("Mollusc"),
    }
}

fn translate_symbol(symbol: &Symbol) -> String {
    match symbol {
        Symbol::Pig => t("Pig"),
        Symbol::Cattle => t("Cattle"),
        Symbol::Poultry => t("Poultry"),
        Symbol::Fish => t("Fish"),
        Symbol::Game => t("Game"),
        Symbol::Lamb => t("Lamb"),
        Symbol::Vegan => t("Vegan"),
        Symbol::Organic => t("Organic"),
        Symbol::Vegetarian => t("Vegetarian"),
        Symbol::Alcohol => t("Alcohol"),
        Symbol::SoupOfTheDay => t("Soup of the Day"),
        Symbol::MensaVital => t("MensaVital"),
        Symbol::Garlic => t("Garlic"),
        Symbol::AnimalWelfare => t("Animal Welfare"),
    }
}

fn icon_name_from_symbol(symbol: &Symbol) -> &'static str {
    match symbol {
        Symbol::Pig => "de.fin_ger.OvGUCanteen.Pig",
        Symbol::Cattle => "de.fin_ger.OvGUCanteen.Cattle",
        Symbol::Poultry => "de.fin_ger.OvGUCanteen.Poultry",
        Symbol::Fish => "de.fin_ger.OvGUCanteen.Fish",
        Symbol::Game => "de.fin_ger.OvGUCanteen.Game",
        Symbol::Lamb => "de.fin_ger.OvGUCanteen.Lamb",
        Symbol::Vegan => "de.fin_ger.OvGUCanteen.Vegan",
        Symbol::Organic => "de.fin_ger.OvGUCanteen.Organic",
        Symbol::Vegetarian => "de.fin_ger.OvGUCanteen.Vegetarian",
        Symbol::Alcohol => "de.fin_ger.OvGUCanteen.Alcohol",
        Symbol::SoupOfTheDay => "de.fin_ger.OvGUCanteen.SoupOfTheDay",
        Symbol::MensaVital => "de.fin_ger.OvGUCanteen.MensaVital",
        Symbol::Garlic => "de.fin_ger.OvGUCanteen.Garlic",
        Symbol::AnimalWelfare => "de.fin_ger.OvGUCanteen.AnimalWelfare",
    }
}

impl MealComponent {
    pub async fn new() -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let meal_box: ListBoxRow = get!(&builder, "meal")?;
        let name: Label = get!(&builder, "meal-name")?;
        let badges: FlowBox = get!(&builder, "badges")?;
        let symbols: FlowBox = get!(&builder, "symbols")?;
        let price_student: Label = get!(&builder, "meal-price-student")?;
        let price_staff: Label = get!(&builder, "meal-price-staff")?;
        let price_guest: Label = get!(&builder, "meal-price-guest")?;

        let symbol_offset = Arc::new(AtomicI32::new(0));
        let allergenic_offset = Arc::new(AtomicI32::new(0));
        let additive_offset = Arc::new(AtomicI32::new(0));

        let symbols = AdjustingVec::new(
            enclose! { (symbols, symbol_offset) move || {
                enclose! { (symbols, symbol_offset) async move {
                    let comp = SymbolComponent::new().await?;
                    symbols.insert(comp.root_widget(), symbol_offset.load(Ordering::SeqCst));
                    symbol_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (symbol_offset) move |badge| {
                enclose! { (symbol_offset) async move {
                    // a flowbox item always has a parent - a FlowBoxChild
                    badge.root_widget().get_parent().unwrap().destroy();
                    symbol_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }}
            }},
        );

        let allergenics = AdjustingVec::new(
            enclose! { (badges, allergenic_offset, additive_offset) move || {
                enclose! { (badges, allergenic_offset, additive_offset) async move {
                    let comp = LiteBadgeComponent::new().await?;
                    badges.insert(comp.root_widget(), allergenic_offset.load(Ordering::SeqCst));
                    allergenic_offset.fetch_add(1, Ordering::SeqCst);
                    additive_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (allergenic_offset, additive_offset) move |badge| {
                enclose! { (allergenic_offset, additive_offset) async move {
                    // a flowbox item always has a parent - a FlowBoxChild
                    badge.root_widget().get_parent().unwrap().destroy();
                    allergenic_offset.fetch_sub(1, Ordering::SeqCst);
                    additive_offset.fetch_sub(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(())
                }}
            }},
        );

        let additives = AdjustingVec::new(
            enclose! { (badges, additive_offset) move || {
                enclose! { (badges, additive_offset) async move {
                    let comp = LiteBadgeComponent::new().await?;
                    badges.insert(comp.root_widget(), additive_offset.load(Ordering::SeqCst));
                    additive_offset.fetch_add(1, Ordering::SeqCst);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            enclose! { (additive_offset) move |badge| {
                enclose! { (additive_offset) async move {
                    // a flowbox item always has a parent - a FlowBoxChild
                    badge.root_widget().get_parent().unwrap().destroy();
                    additive_offset.fetch_sub(1, Ordering::SeqCst);

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
                badge.load(&translate_additive(&additive)).await;
                glib_yield!();
                Ok(badge)
            })
            .await?;

        self.allergenics
            .adjust(&meal.allergenics, |badge, allergenic| async move {
                badge.load(&translate_allergenic(&allergenic)).await;
                glib_yield!();
                Ok(badge)
            })
            .await?;

        self.symbols
            .adjust(&meal.symbols, |badge, symbol| async move {
                badge.load(icon_name_from_symbol(&symbol), &translate_symbol(&symbol)).await;
                glib_yield!();
                Ok(badge)
            })
            .await?;

        Ok(())
    }
}

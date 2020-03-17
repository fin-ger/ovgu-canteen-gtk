use anyhow::{bail, Context, Result};
use cargo_author::Author;
use chrono::{Datelike, TimeZone, Utc, Weekday};
use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;
use gtk::{
    Box, Builder, ButtonRole, FlowBox, Frame, Label, ListBox, ListBoxRow, MenuButton,
    ModelButtonBuilder, Spinner, Stack, Window, AboutDialog, Button,
};
use tokio::sync::mpsc::channel;
use tokio::runtime::Handle;
use ovgu_canteen::{Canteen, CanteenDescription, Day, Error as CanteenError, Meal};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

pub const GLADE: &str = std::include_str!("../data/gnome-ovgu-canteen.glade");

macro_rules! glib_yield {
    () => {
        glib::timeout_future_with_priority(glib::PRIORITY_DEFAULT_IDLE, 0).await
    };
}

#[inline]
pub fn get<T: IsA<glib::Object>>(builder: &Builder, id: &str) -> Result<T> {
    builder
        .get_object(id)
        .context(format!("'{}' is not available in glade file", id))
}

#[derive(Debug)]
pub struct WindowComponent {
    pub window: Window,
    pub canteens_stack: Rc<RefCell<Stack>>,
    pub canteens_menu: Box,
    pub canteen_menu_button: MenuButton,
    pub canteen_label: Rc<RefCell<Label>>,
    pub reload_button: Rc<RefCell<Button>>,
    pub canteen_components: Rc<RefCell<HashMap<CanteenDescription, CanteenComponent>>>,
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

impl WindowComponent {
    pub fn new(rt: &Handle, app: &gtk::Application) -> Result<()> {
        let builder = Builder::new_from_string(GLADE);

        let mut canteens = vec![
            CanteenDescription::UniCampusLowerHall,
            CanteenDescription::UniCampusUpperHall,
            CanteenDescription::Kellercafe,
            CanteenDescription::Herrenkrug,
            CanteenDescription::Stendal,
            CanteenDescription::Wernigerode,
            CanteenDescription::DomCafeteHalberstadt,
        ];
        let window: Window = get(&builder, "window")?;
        let canteens_stack: Rc<RefCell<Stack>> = Rc::new(RefCell::new(get(&builder, "canteens-stack")?));
        let canteens_menu: Box = get(&builder, "canteens-menu")?;
        let canteen_label: Rc<RefCell<Label>> = Rc::new(RefCell::new(get(&builder, "canteen-label")?));
        let canteen_menu_button: MenuButton = get(&builder, "canteen-menu-button")?;
        let about_dialog: AboutDialog = get(&builder, "about")?;
        let about_button: Button = get(&builder, "about-btn")?;
        let options_button: MenuButton = get(&builder, "options-button")?;
        let reload_button: Rc<RefCell<Button>> = Rc::new(RefCell::new(get(&builder, "reload-button")?));

        window.set_application(Some(app));
        window.set_icon_name(Some("ovgu-canteen32"));
        about_dialog.set_logo_icon_name(Some("ovgu-canteen128"));

        let authors = env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(|author| Author::new(author))
            .collect::<Vec<_>>();

        about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
        about_dialog.set_website(Some(env!("CARGO_PKG_REPOSITORY")));
        about_dialog.set_website_label(Some("Source Code"));
        about_dialog.set_comments(Some(env!("CARGO_PKG_DESCRIPTION")));
        about_dialog.set_authors(
            &authors
                .iter()
                .map(|author| {
                    if let Some(name) = &author.name {
                        Ok(name.as_str())
                    } else if let Some(email) = &author.email {
                        Ok(email.as_str())
                    } else if let Some(url) = &author.url {
                        Ok(url.as_str())
                    } else {
                        bail!("Failed to get author name");
                    }
                })
                .collect::<Result<Vec<_>>>()?,
        );
        about_button.connect_clicked(move |_btn| {
            if let Some(popover) = options_button.get_popover() {
                popover.popdown();
            }

            about_dialog.run();
            about_dialog.hide();
        });

        window.show_all();

        let comp = Self {
            window,
            canteens_stack,
            canteens_menu,
            canteen_label,
            canteen_menu_button,
            reload_button,
            canteen_components: Rc::new(RefCell::new(HashMap::new())),
        };

        let mut canteen_components_borrow = comp.canteen_components.borrow_mut();
        for desc in canteens.drain(..) {
            canteen_components_borrow.insert(
                desc.clone(),
                CanteenComponent::new(desc, &comp)
                    .context("Failed to create canteen!")?,
            );
        }
        drop(canteen_components_borrow);

        comp.load(rt);

        let reload_rt = rt.clone();
        comp.reload_button.clone().borrow().connect_clicked(move |_btn| {
            comp.load(&reload_rt);
        });

        Ok(())
    }

    pub fn load(&self, rt: &Handle) {
        self.reload_button.borrow().set_sensitive(false);

        // canteens are downloaded in parallel here,
        // but in order for one canteen to show up in a batch
        // we are using an mpsc channel to put the parallel loaded canteens
        // in an order which is later sequentially inserted into the GUI.
        let (tx, mut rx) = channel(self.canteen_components.borrow().len());
        for (canteen_desc_ref, _comp) in self.canteen_components.borrow().iter() {
            let mut canteen_tx = tx.clone();
            let canteen_desc = canteen_desc_ref.clone();
            rt.spawn(async move {
                let canteen = Canteen::new(canteen_desc.clone()).await;
                if let Err(e) = canteen_tx.send((canteen_desc, canteen)).await {
                    eprintln!("error: {}", e);
                    // TODO: handle tx send error by displaying canteen not available
                }
            });
        }

        let c = glib::MainContext::default();
        let fetch_reload_button = self.reload_button.clone();
        let fetch_canteen_components = self.canteen_components.clone();
        c.spawn_local(async move {
            // fetching parallel loaded canteens here and inserting
            // one canteen after another into the GUI.
            // TODO: render currently visible canteen first
            while let Some((desc, canteen)) = rx.recv().await {
                if let Some(comp) = fetch_canteen_components.borrow().get(&desc) {
                    comp.load(canteen).await;
                } else {
                    eprintln!("canteen {:?} not found in components list", desc);
                    // TODO: display error dialog
                }
            }

            fetch_reload_button.borrow().set_sensitive(true);
        });
    }
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

        window
            .canteens_stack
            .borrow_mut()
            .add_named(&canteen_stack, &canteen_name);

        let model_btn = ModelButtonBuilder::new()
            .visible(true)
            .text(&canteen_name)
            .can_focus(false)
            .action_name(&format!("app.{}", canteen_name))
            .role(ButtonRole::Radio)
            .build();

        window.canteens_menu.pack_start(&model_btn, false, true, 0);

        let action = SimpleAction::new(&canteen_name, None);
        let canteens_stack_handle = Rc::clone(&window.canteens_stack);
        let canteen_label_handle = Rc::clone(&window.canteen_label);
        action.connect_activate(move |_action, _variant| {
            canteens_stack_handle
                .borrow()
                .set_visible_child_name(&canteen_name);
            canteen_label_handle.borrow().set_text(&canteen_name);
        });
        window.window.get_application()
            .context("GTK Application not initialized!")?
            .add_action(&action);

        Ok(Self {
            canteen_stack,
            canteen_spinner,
            days_box,
            description,
        })
    }

    pub async fn load(&self, load_result: Result<Canteen, CanteenError>) {
        let mut canteen = match load_result {
            Ok(canteen) => canteen,
            Err(e) => {
                eprintln!("error: {}", e);
                self.canteen_stack.set_visible_child_name("canteen-error");
                // TODO: display error message
                return;
            }
        };

        for day in canteen.days.drain(..) {
            match DayComponent::new(&day).await {
                Ok(day_comp) => {
                    self.days_box.pack_start(&day_comp.frame, false, true, 0);
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    // TODO: add error handling for failed daycomponent
                    continue;
                }
            }

            glib_yield!();
        }

        self.canteen_spinner.stop();
        self.canteen_spinner.hide();
    }
}

impl DayComponent {
    pub async fn new(day: &Day) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let frame: Frame = get(&builder, "day-frame")?;
        let label: Label = get(&builder, "day-label")?;
        let meals_list_box: ListBox = get(&builder, "day-meals-list-box")?;
        let side_dish_badges: FlowBox = get(&builder, "side-dish-badges")?;

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
        let meal_box: ListBoxRow = get(&builder, "meal")?;
        let name: Label = get(&builder, "meal-name")?;
        let badges: FlowBox = get(&builder, "badges")?;
        let price_student: Label = get(&builder, "meal-price-student")?;
        let price_staff: Label = get(&builder, "meal-price-staff")?;
        let price_guest: Label = get(&builder, "meal-price-guest")?;

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
        let label: Label = Label::new(Some(text));
        let context = label.get_style_context();
        context.add_class("badge");
        label.set_selectable(true);
        label.set_line_wrap(false);
        label.set_visible(true);

        Ok(Self { label })
    }
}

impl LiteBadgeComponent {
    pub async fn new(text: &str) -> Result<Self> {
        let label: Label = Label::new(Some(text));
        let context = label.get_style_context();
        context.add_class("badge-lite");
        label.set_selectable(true);
        label.set_line_wrap(false);
        label.set_visible(true);

        Ok(Self { label })
    }
}

use anyhow::Result;
use gtk::prelude::*;
use gtk::{Label, Image, IconSize};

pub struct BadgeComponent {
    label: Label,
}

pub struct LiteBadgeComponent {
    label: Label,
}

pub struct SymbolComponent {
    image: Image,
}

impl BadgeComponent {
    pub async fn new() -> Result<Self> {
        log::debug!("new BadgeComponent created");

        let label = Label::new(None);
        let context = label.get_style_context();
        // css class 'badge' styles this component
        context.add_class("badge");
        label.set_selectable(true);
        label.set_line_wrap(false);
        label.set_visible(true);

        Ok(Self { label })
    }

    pub const fn root_widget(&self) -> &Label {
        &self.label
    }

    pub async fn load(&self, text: &str) {
        log::debug!("loading content into BadgeComponent: {}", text);

        self.label.set_text(text);
    }
}

impl LiteBadgeComponent {
    pub async fn new() -> Result<Self> {
        log::debug!("new LiteBadgeComponent created");

        let label = Label::new(None);
        let context = label.get_style_context();
        // css class 'badge-lite' styles this component
        context.add_class("badge-lite");
        label.set_selectable(true);
        label.set_line_wrap(false);
        label.set_visible(true);

        Ok(Self { label })
    }

    pub const fn root_widget(&self) -> &Label {
        &self.label
    }

    pub async fn load(&self, text: &str) {
        log::debug!("loading content into LiteBadgeComponent: {}", text);

        self.label.set_text(text);
    }
}

impl SymbolComponent {
    pub async fn new() -> Result<Self> {
        log::debug!("new SymbolComponent created");

        let image = Image::new();
        image.set_visible(true);

        Ok(Self { image })
    }

    pub const fn root_widget(&self) -> &Image {
        &self.image
    }

    pub async fn load(&self, name: &str, tooltip: &str) {
        log::debug!("loading content into SymbolComponent: (icon-name: {}, tooltip: {})", name, tooltip);

        // symbols are installed as system icons (from ./icons), therefore they are available as icon-names
        self.image.set_from_icon_name(Some(name), IconSize::LargeToolbar);
        self.image.set_tooltip_text(Some(tooltip));
    }
}

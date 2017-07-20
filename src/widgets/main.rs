use ::gtk;

use application::Application;
use std::sync::Arc;
use gtk::prelude::*;

pub struct MainWidget
{
    pub horizontal_navigation_stack: gtk::Stack,
    pub back_button: gtk::Button,
    pub canteen_list_box: gtk::ListBox,
}

impl MainWidget
{
    pub fn canteen_activated(&self, _app: &Application, _list_box: &gtk::ListBox, _row: &gtk::ListBoxRow)
    {
        self.horizontal_navigation_stack.set_visible_child_name("menu");
        self.back_button.set_sensitive(true);
    }

    pub fn back_clicked(&self, _app: &Application, _button: &gtk::Button)
    {
        self.horizontal_navigation_stack.set_visible_child_name("canteens");
        self.back_button.set_sensitive(false);
    }

    pub fn connect_signals(&self, app: &Arc<Application>)
    {
        {
            let binding = app.clone();
            app.widgets.main_widget.canteen_list_box.connect_row_activated(move |list_box, row| {
                binding.widgets.main_widget.canteen_activated(&binding, list_box, row);
            });
        }

        {
            let binding = app.clone();
            app.widgets.main_widget.back_button.connect_clicked(move |button| {
                binding.widgets.main_widget.back_clicked(&binding, button);
            });
        }
    }
}

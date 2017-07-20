use gio_sys::{GAction, GActionEntry, GSimpleAction};
use ::gio_sys;
use glib::translate::ToGlibPtr;
use glib_sys::{GVariant, gpointer};
use ::gtk;
use libc::c_void;
use std::ffi::CStr;
use application::Application;

type ActionCallback = unsafe extern "C" fn(*mut GSimpleAction, *mut GVariant, *mut c_void);

pub struct ActionEntry<'a>
{
    name: &'a str,
    parameter_type: Option<&'a str>,
    state: Option<&'a str>,
    activate: Option<ActionCallback>,
    change_state: Option<ActionCallback>,
}

impl<'a> ActionEntry<'a>
{
    pub fn new<P, S>(
        name: &'a str,
        activate: Option<ActionCallback>,
        parameter_type: P,
        change_state: Option<ActionCallback>,
        state: S
    ) -> ActionEntry<'a>
        where P: Into<Option<&'a str>>, S: Into<Option<&'a str>>
    {
        ActionEntry
        {
            name: name,
            activate: activate.into(),
            parameter_type: parameter_type.into(),
            state: state.into(),
            change_state: change_state.into(),
        }
    }
}

pub fn g_action_map_add_action_entries<'a, I, T>(gapp: &gtk::Application, entries: I, user_data: &T)
    where I: Iterator<Item=&'a ActionEntry<'a>>
{
    // cache the stashes from `to_glib_none` to avoid them going out of scope
    let mut names = vec![];
    let mut params = vec![];
    let mut states = vec![];

    let mut vec = entries.map(|entry| {
        names.push(entry.name.to_glib_none());
        params.push(entry.parameter_type.to_glib_none());
        states.push(entry.state.to_glib_none());

        GActionEntry
        {
            name: names.last().unwrap().0,
            activate: entry.activate,
            parameter_type: params.last().unwrap().0,
            state: states.last().unwrap().0,
            change_state: entry.change_state,
            padding: [0, 0, 0],
        }
    }).collect::<Vec<GActionEntry>>();
    let slice = vec.as_mut_slice();

    unsafe
    {
        gio_sys::g_action_map_add_action_entries(
            gapp.to_glib_none().0,
            slice.as_mut_ptr(),
            slice.len() as i32,
            user_data as *const _ as *mut c_void
        );
    };
}

pub unsafe extern fn action_entry_activate(
    action: *mut GSimpleAction,
    _: *mut GVariant,
    user_data: gpointer
)
{
    let cname = CStr::from_ptr(
        gio_sys::g_action_get_name(action as *mut GAction) as *const _
    );
    let app = (user_data as *const Application).as_ref().unwrap();

    match cname.to_str().unwrap()
    {
        "prefs" => app.widgets.app_menu_widget.prefs(app),
        "about" => app.widgets.app_menu_widget.about(app),
        "quit" => app.widgets.app_menu_widget.quit(app),
        _ => println!("No handler registered for action '{}'!", cname.to_str().unwrap()),
    };
}

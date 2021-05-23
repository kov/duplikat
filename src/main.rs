#[macro_use]
extern crate gtk_macros;

use gettextrs::*;
use gtk::prelude::*;

mod config;
mod widgets;
mod window;
use crate::window::{Window, WindowViews};

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("duplikat", config::LOCALEDIR);
    textdomain("duplikat");

    libadwaita::init();

    let app = gtk::Application::new(Some("br.dev.kov.Duplikat"), Default::default());

    app.connect_activate(move |app| {
        let window = Window::new();

        window.widget.set_application(Some(app));
        app.add_window(&window.widget);

        // win.add
        let w = window.clone();
        action!(
            window.widget,
            "add",
            move |_, _| {
                w.set_view(WindowViews::Create);
            }
        );

        // win.add-finish
        let w = window.clone();
        action!(
            window.widget,
            "add-finish",
            move |_, _| {
                w.create_backup();
            }
        );

        // win.go-previous
        let w = window.clone();
        action!(
            window.widget,
            "go-previous",
            move |_, _| {
                w.set_view(WindowViews::List);
            }
        );

        app.set_accels_for_action("win.add", &[]);
        app.set_accels_for_action("win.go-previous", &["Escape"]);

        window.widget.present();
    });

    let ret = app.run();
    std::process::exit(ret);
}

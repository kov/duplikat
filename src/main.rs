#[macro_use]
extern crate gtk_macros;

use gettextrs::*;
use gtk::prelude::*;

mod config;
mod widgets;
mod window;
use crate::window::Window;

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("duplikat", config::LOCALEDIR);
    textdomain("duplikat");

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/duplikat.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

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
            move |action, _| {
                w.create_backup();
            }
        );

        app.set_accels_for_action("win.add", &[]);

        window.widget.present();
    });

    let ret = app.run();
    std::process::exit(ret);
}

use std::cell::RefCell;
use std::rc::Rc;
use glib::clone;
use gtk::prelude::*;
use gtk::ApplicationWindow;

mod server;
mod edit;
mod overview;

thread_local!(
    static WINDOW: RefCell<Option<Rc<gtk::ApplicationWindow>>> = RefCell::new(None);
);

pub fn get_main_window() -> gtk::ApplicationWindow {
    let mut window: Option<gtk::ApplicationWindow> = None;
    WINDOW.with(|w| {
        if let Some(w) = w.borrow().as_ref() {
            window.replace((**w).clone());
        }
    });
    window.expect("Main window not initialized!")
}

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_resource("/br/dev/kov/Duplikat/style.css");
    gtk::StyleContext::add_provider_for_display(
	&gdk::Display::default().unwrap(),
	&css_provider,
	800
    );

    let app = gtk::Application::new(Some("br.dev.kov.Duplikat"), Default::default());
    app.connect_activate(move |app| {
        let window = create_ui(app);
        window.present();
        WINDOW.with(|w| *w.borrow_mut() = Some(Rc::new(window)));
    });

    let ret = app.run();
    std::process::exit(ret);
}

fn create_ui(app: &gtk::Application) -> ApplicationWindow {
    let window = gtk::ApplicationWindow::new(app);
    window.set_title(Some("Duplikat"));

    // Headerbar
    let headerbar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&headerbar));

    let create_button = gtk::ButtonBuilder::new()
        .icon_name("list-add-symbolic")
        .build();
    headerbar.pack_start(&create_button);

    let back_button = gtk::ButtonBuilder::new()
        .icon_name("go-previous-symbolic")
        .build();
    back_button.set_visible(false);
    headerbar.pack_start(&back_button);

    let stack = gtk::Stack::new();
    window.set_child(Some(&stack));

    create_button.connect_clicked(
        clone!(@weak stack, @weak create_button, @weak back_button => move |_| {
            create_button.set_visible(false);
            back_button.set_visible(true);
            stack.set_visible_child_name("create/edit");
        })
    );

    back_button.connect_clicked(
        clone!(@weak stack, @weak create_button, @weak back_button => move |_| {
            create_button.set_visible(true);
            back_button.set_visible(false);
            stack.set_visible_child_name("overview");
        })
    );

    // Backups list
    let overview = overview::OverviewUI::new();
    stack.add_titled(&overview.borrow().container, Some("overview"), "Backups Overview");

    // Create/edit backup
    let create_edit = edit::CreateEditUI::new();
    stack.add_titled(&create_edit.borrow().container, Some("create/edit"), "Create or edit");

    window
}

use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;
use crate::prefix::PREFIX;

mod server;
mod edit;
mod overview;
mod prefix;
mod utils;

pub struct Application {
    pub application: gtk::Application,
    pub main_window: gtk::ApplicationWindow,
    pub stack: gtk::Stack,
    pub create_button: gtk::Button,
    pub overview: Option<Rc<RefCell<overview::OverviewUI>>>,
    pub create_edit: Option<Rc<RefCell<edit::CreateEditUI>>>,
}

impl Application {
    fn new(application: gtk::Application, main_window: gtk::ApplicationWindow, stack: gtk::Stack,
        create_button: gtk::Button) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(
            Application {
                application,
                main_window,
                stack,
                create_button,
                overview: None,
                create_edit: None,
            }
        ))
    }

    fn open_create_edit(&self) {
        self.create_edit.as_ref().unwrap().borrow().open();
    }

    fn update(&mut self) {
        self.overview.as_mut().unwrap().borrow_mut().update();
    }
}

fn main() {
    dbg!("{}", PREFIX);
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
        create_ui(app);
    });

    let ret = app.run();
    std::process::exit(ret);
}

fn create_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(app);
    window.set_title(Some("Duplikat"));

    // Headerbar
    let headerbar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&headerbar));

    let create_button = gtk::ButtonBuilder::new()
        .icon_name("list-add-symbolic")
        .build();
    headerbar.pack_start(&create_button);

    let stack = gtk::Stack::new();
    window.set_child(Some(&stack));

    let application = Application::new(
        app.clone(),
        window.clone(),
        stack.clone(),
        create_button.clone(),
    );

    let app = application.clone();
    create_button.connect_clicked(move |_| {
        app.borrow().open_create_edit();
    });

    // Backups list
    let overview = overview::OverviewUI::new(application.clone());
    application.borrow_mut().overview.replace(overview.clone());
    stack.add_titled(&overview.borrow().container, Some("overview"), "Backups Overview");

    // Create/edit backup
    let create_edit = edit::CreateEditUI::new(application.clone());
    application.borrow_mut().create_edit.replace(create_edit.clone());

    window.present();
}

use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;

mod server;
mod edit;
mod overview;

pub enum StackPage {
    Overview,
    CreateEdit,
}

pub struct Application {
    pub main_window: gtk::ApplicationWindow,
    pub stack: gtk::Stack,
    pub create_button: gtk::Button,
    pub back_button: gtk::Button,
}

impl Application {
    fn new(main_window: gtk::ApplicationWindow, stack: gtk::Stack,
        create_button: gtk::Button, back_button: gtk::Button) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(
            Application {
                main_window,
                stack,
                create_button,
                back_button,
            }
        ))
    }

    fn set_stack_page(&mut self, page: StackPage) {
        match page {
            StackPage::Overview => {
                self.stack.set_visible_child_name("overview");
                self.create_button.set_visible(true);
                self.back_button.set_visible(false);
            },
            StackPage::CreateEdit => {
                self.stack.set_visible_child_name("create/edit");
                self.create_button.set_visible(false);
                self.back_button.set_visible(true);
            },
        }
    }
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

    let back_button = gtk::ButtonBuilder::new()
        .icon_name("go-previous-symbolic")
        .build();
    back_button.set_visible(false);
    headerbar.pack_start(&back_button);

    let stack = gtk::Stack::new();
    window.set_child(Some(&stack));

    let application = Application::new(
        window.clone(),
        stack.clone(),
        create_button.clone(),
        back_button.clone(),
    );

    let app = application.clone();
    create_button.connect_clicked(move |_| {
        app.borrow_mut().set_stack_page(StackPage::CreateEdit);
    });

    let app = application.clone();
    back_button.connect_clicked(move |_| {
        app.borrow_mut().set_stack_page(StackPage::Overview);
    });

    // Backups list
    let overview = overview::OverviewUI::new(application.clone());
    stack.add_titled(&overview.borrow().container, Some("overview"), "Backups Overview");

    // Create/edit backup
    let create_edit = edit::CreateEditUI::new(application.clone());
    stack.add_titled(&create_edit.borrow().container, Some("create/edit"), "Create or edit");

    window.present();
}

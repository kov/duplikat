#![feature(async_closure)]

use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::path::PathBuf;
use glib::{MainContext, clone};
use gtk::prelude::*;
use gtk::ApplicationWindow;
use duplikat_types::*;
use strum::IntoEnumIterator;
use crate::server::Server;

mod server;

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

    let app = gtk::Application::new(Some("br.dev.kov.Duplikat"), Default::default());
    app.connect_activate(move |app| {
        let window = create_ui(app);
        window.present();
        WINDOW.with(|w| *w.borrow_mut() = Some(Rc::new(window)));
    });

    let ret = app.run();
    std::process::exit(ret);
}

fn create_backups_list_ui() -> gtk::ListBox {
    let listbox = gtk::ListBox::new();
    listbox.set_css_classes(&["rich-list"]);

    MainContext::default().spawn_local(
        async {
            let connection = match Server::connect().await {
                Ok(c) => c,
                Err(_) => return,
            };

            if let Err(error) = connection.send_message(ClientMessage::ListBackups).await {
                println!("Error listing backups: {:#?}", error);
            };

            while let Ok(message) = connection.read_message().await {
                if let Some(message) = message {
                    match message {
                        ResticMessage::BackupsList(list) => {
                            dbg!(list);
                        },
                        _ => unimplemented!(),
                    }
                } else {
                    break;
                }
            }
        }
    );

    listbox
}

fn create_ui(app: &gtk::Application) -> ApplicationWindow {
    let window = gtk::ApplicationWindow::new(app);
    window.set_title(Some("Duplikat"));

    // Headerbar
    let headerbar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&headerbar));

    let stack_switcher = gtk::StackSwitcher::new();
    headerbar.pack_start(&stack_switcher);

    let stack = gtk::Stack::new();
    window.set_child(Some(&stack));
    stack_switcher.set_stack(Some(&stack));

    // Backups list
    let backups_listbox = create_backups_list_ui();
    stack.add_titled(&backups_listbox, Some("backups"), "Backups list");

    // Create/edit backup
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    stack.add_titled(&hbox, Some("create/edit"), "Create or edit");

    let grid = gtk::Grid::new();

    hbox.append(&grid);

    // Name
    let label = gtk::Label::new(Some("Name"));
    grid.attach(&label, 0, 0, 1, 1);

    let name_entry = gtk::Entry::new();
    name_entry.set_text("kov");
    name_entry.set_placeholder_text(Some("System backup"));

    grid.attach_next_to(&name_entry, Some(&label), gtk::PositionType::Right, 1, 1);

    // Type
    let label = gtk::Label::new(Some("Type"));
    grid.attach(&label, 0, 1, 1, 1);

    let type_combo = gtk::ComboBoxText::new();
    for kind in RepositoryKind::iter() {
        type_combo.append(Some(&kind.to_string()), kind.to_human_readable());
    }
    type_combo.set_active_id(Some("local"));

    grid.attach_next_to(&type_combo, Some(&label), gtk::PositionType::Right, 1, 1);

    // Identifier (bucket name, host, etc)
    let label = gtk::Label::new(None);
    label.set_visible(false);
    grid.attach(&label, 0, 2, 1, 1);

    let identifier = gtk::Entry::new();
    identifier.set_visible(false);

    grid.attach_next_to(&identifier, Some(&label), gtk::PositionType::Right, 1, 1);

    // Adjust entries based on type
    type_combo.connect_changed(
        clone!(@weak type_combo, @weak label, @weak identifier => move |_| {
            let repo_type = type_combo.active_id()
                .expect("Combo box should never be empty")
                .to_string();

            // Most types will need an identifier
            label.set_visible(true);
            identifier.set_visible(true);
            match RepositoryKind::from_str(repo_type.as_str()).unwrap() {
                RepositoryKind::Local => {
                    label.set_visible(false);
                    identifier.set_visible(false);
                },
                RepositoryKind::B2 => {
                    label.set_label("Bucket");
                    identifier.set_placeholder_text(Some("bucket-name"));
                },
                RepositoryKind::SFTP => {
                    label.set_label("Host");
                    identifier.set_placeholder_text(Some("[user@]host-or-ip.com"));
                },
            }
        })
    );

    // Path
    let label = gtk::Label::new(Some("Path"));
    grid.attach(&label, 0, 2, 1, 1);

    let path = gtk::Entry::new();
    path.set_text("/tmp/duplikat-dev");

    grid.attach_next_to(&path, Some(&label), gtk::PositionType::Right, 1, 1);

    // Password
    let label = gtk::Label::new(Some("Password"));
    grid.attach(&label, 0, 3, 1, 1);

    let password = gtk::PasswordEntry::new();
    password.set_text("lala");
    password.set_show_peek_icon(true);

    grid.attach_next_to(&password, Some(&label), gtk::PositionType::Right, 1, 1);

    // Password
    let label = gtk::Label::new(Some("Confirm Password"));
    grid.attach(&label, 0, 4, 1, 1);

    let confirm = gtk::PasswordEntry::new();
    confirm.set_text("lala");
    confirm.set_show_peek_icon(true);

    grid.attach_next_to(&confirm, Some(&label), gtk::PositionType::Right, 1, 1);

    let add_backup = gtk::Button::with_label("Add Backup");
    add_backup.set_sensitive(true);

    // Disable or enable add_backup based on password
    confirm.connect_changed(
        clone!(@weak add_backup, @weak password, @weak confirm => move |_| {
            let password = password.text().to_string();
            let confirm = confirm.text().to_string();
            if password != confirm {
                add_backup.set_sensitive(false);
             } else {
                add_backup.set_sensitive(true);
            }
        })
    );

    add_backup.connect_clicked(
        clone!(@weak name_entry => move |_| {
            let repo_type = type_combo.active_id()
                .expect("Combo box should never be empty")
                .to_string();
            let identifier = identifier.text().to_string();
            let path = path.text().to_string();
            let password = password.text().to_string();

            let repository_str = format!("{}:{}:{}", repo_type, identifier, path);

            let backup = Backup {
                name: name_entry.text().to_string(),
                repository: Repository::from(repository_str.as_str()),
                password,
                include: vec![PathBuf::from("/home/kov/Downloads"), PathBuf::from("/home/kov/Projects/gbuild")],
                exclude: vec![".cache".to_string()],
            };

            MainContext::default().spawn_local(async move {
                let connection = match Server::connect().await {
                    Ok(c) => c,
                    Err(_) => return,
                };

                if let Err(error) = connection.send_message(
                    ClientMessage::CreateBackup(ClientMessageCreateBackup {backup})
                ).await { println!("Error creating backup: {:#?}", error) }
            });
         })
    );

    grid.attach(&add_backup, 1, 5, 1, 1);

    // List backups view
    let listbox = gtk::ListBox::new();
    stack.add_titled(&listbox, Some("list/run"), "List & run");

    let row = gtk::ListBoxRow::new();
    row.set_selectable(false);
    listbox.append(&row);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    row.set_child(Some(&hbox));

    let run_button = gtk::Button::with_label("Run");
    hbox.append(&run_button);

    let progress_bar = gtk::ProgressBar::new();
    progress_bar.set_valign(gtk::Align::Center);
    progress_bar.set_halign(gtk::Align::Fill);
    hbox.append(&progress_bar);

    run_button.connect_clicked(
        clone!(@weak name_entry, @weak progress_bar => move |_| {
            MainContext::default().spawn_local(async move {
                let run_backup_message = ClientMessage::RunBackup(
                    ClientMessageRunBackup {
                        name: name_entry.text().to_string()
                    }
                );

                let connection = match Server::connect().await {
                    Ok(c) => c,
                    Err(_) => return,
                };

                if let Err(error) = connection.send_message(run_backup_message).await {
                    println!("Failed to run...: {:#?}", error);
                    return;
                };

                while let Ok(message) = connection.read_message().await {
                    if let Some(message) = message {
                        match message {
                            ResticMessage::Status(status) => {
                                progress_bar.set_fraction(status.percent_done);
                            },
                            ResticMessage::Summary(_) => {
                                progress_bar.set_fraction(1.);
                            },
                            _ => unimplemented!(),
                        }
                    } else {
                        break;
                    }
                }
            });
        })
    );

    window
}
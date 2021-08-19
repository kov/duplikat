use std::{cell::RefCell, rc::Rc};
use std::str::FromStr;
use std::path::PathBuf;
use glib::{MainContext, clone};
use gtk::prelude::*;
use duplikat_types::*;
use strum::IntoEnumIterator;
use crate::Application;
use crate::server::Server;

pub struct CreateEditUI {
    pub window: gtk::Dialog,
    myself: Option<Rc<RefCell<Self>>>,
    name: gtk::Entry,
    kind: gtk::ComboBoxText,
    identifier: gtk::Entry,
    key_id: gtk::Entry,
    key_secret: gtk::Entry,
    path: gtk::Entry,
    password: gtk::PasswordEntry,
    confirm: gtk::PasswordEntry,
    include: Vec<gtk::Editable>,
    exclude: Vec<gtk::Editable>,
    stack: gtk::Stack,
    back_button: gtk::Button,
    forward_button: gtk::Button,
    add_backup: gtk::Button,
}

enum Go {
    Back,
    Forward,
}

fn next_row_num(num: &mut i32) -> i32 {
    *num += 1;
    *num
}

impl CreateEditUI {
    pub(crate) fn new(application: Rc<RefCell<Application>>) -> Rc<RefCell<Self>> {
        let window = gtk::DialogBuilder::new()
            .transient_for(&application.borrow().main_window)
            .hide_on_close(true)
            .use_header_bar(1)
            .modal(true)
            .build();

        let stack = gtk::Stack::new();
        window.set_child(Some(&stack));

        // Repository details
        let grid = gtk::Grid::new();
        stack.add_titled(&grid, Some("repository"), "Repository");

        let mut row_num = -1i32;

        // Name
        let label = gtk::Label::new(Some("Name"));
        grid.attach(&label, next_row_num(&mut row_num), 0, 1, 1);

        let name_entry = gtk::Entry::new();
        name_entry.set_placeholder_text(Some("System backup"));

        grid.attach_next_to(&name_entry, Some(&label), gtk::PositionType::Right, 1, 1);

        // Type
        let label = gtk::Label::new(Some("Type"));
        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let type_combo = gtk::ComboBoxText::new();
        for kind in RepositoryKind::iter() {
            type_combo.append(Some(&kind.to_string()), kind.to_human_readable());
        }
        type_combo.set_active_id(Some("local"));

        grid.attach_next_to(&type_combo, Some(&label), gtk::PositionType::Right, 1, 1);

        // Identifier (bucket name, host, etc)
        let label = gtk::Label::new(None);
        label.set_visible(false);
        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let identifier = gtk::Entry::new();
        identifier.set_visible(false);

        grid.attach_next_to(&identifier, Some(&label), gtk::PositionType::Right, 1, 1);

        // Key ID
        let key_label = gtk::Label::new(Some("Key ID"));
        key_label.set_visible(false);

        grid.attach(&key_label, 0, next_row_num(&mut row_num), 1, 1);

        let key_entry = gtk::Entry::new();
        key_entry.set_visible(false);

        grid.attach_next_to(&key_entry, Some(&key_label), gtk::PositionType::Right, 1, 1);

        // Key Secret
        let secret_label = gtk::Label::new(Some("Secret ID"));
        secret_label.set_visible(false);

        grid.attach(&secret_label, 0, next_row_num(&mut row_num), 1, 1);

        let secret_entry = gtk::Entry::new();
        secret_entry.set_visible(false);

        grid.attach_next_to(&secret_entry, Some(&secret_label), gtk::PositionType::Right, 1, 1);

        // Adjust entries based on type
        type_combo.connect_changed(
            clone!(@weak type_combo, @weak label, @weak identifier,
                   @weak key_label, @weak key_entry,
                   @weak secret_label, @weak secret_entry => move |_| {
                let repo_type = type_combo.active_id()
                    .expect("Combo box should never be empty")
                    .to_string();

                // Most types will need an identifier / host
                label.set_visible(true);
                identifier.set_visible(true);

                // For now most types do not need key / secret
                key_label.set_visible(false);
                key_entry.set_visible(false);

                secret_label.set_visible(false);
                secret_entry.set_visible(false);

                match RepositoryKind::from_str(repo_type.as_str()).unwrap() {
                    RepositoryKind::Local => {
                        label.set_visible(false);
                        identifier.set_visible(false);
                    },
                    RepositoryKind::B2 => {
                        label.set_label("Bucket");
                        identifier.set_placeholder_text(Some("bucket-name"));

                        key_label.set_visible(true);
                        key_entry.set_visible(true);
                        key_entry.set_placeholder_text(Some("key-id"));

                        secret_label.set_visible(true);
                        secret_entry.set_visible(true);
                        secret_entry.set_placeholder_text(Some("secret-id"));
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
        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let path = gtk::Entry::new();

        grid.attach_next_to(&path, Some(&label), gtk::PositionType::Right, 1, 1);

        // Password
        let label = gtk::Label::new(Some("Password"));
        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let password = gtk::PasswordEntry::new();
        password.set_show_peek_icon(true);

        grid.attach_next_to(&password, Some(&label), gtk::PositionType::Right, 1, 1);

        // Password
        let label = gtk::Label::new(Some("Confirm Password"));
        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let confirm = gtk::PasswordEntry::new();
        confirm.set_show_peek_icon(true);

        grid.attach_next_to(&confirm, Some(&label), gtk::PositionType::Right, 1, 1);

        // Add buttons, and signal handlers.
        let headerbar = window.header_bar();

        let back_button = gtk::Button::with_label("Back");
        back_button.set_sensitive(false);
        headerbar.pack_start(&back_button);

        let forward_button = gtk::Button::with_label("Forward");
        forward_button.set_css_classes(&["suggested-action"]);
        forward_button.set_sensitive(false);
        headerbar.pack_end(&forward_button);

        let add_backup = gtk::Button::with_label("Add Backup");
        add_backup.set_css_classes(&["suggested-action"]);
        add_backup.set_visible(false);
        add_backup.set_sensitive(false);
        headerbar.pack_end(&add_backup);

        let myself = Rc::new(RefCell::new(
            CreateEditUI {
                window: window.clone(),
                myself: None,
                name: name_entry.clone(),
                identifier: identifier.clone(),
                kind: type_combo.clone(),
                key_id: key_entry.clone(),
                key_secret: secret_entry.clone(),
                path: path.clone(),
                password: password.clone(),
                confirm: confirm.clone(),
                include: vec![],
                exclude: vec![],
                stack: stack.clone(),
                back_button: back_button.clone(),
                forward_button: forward_button.clone(),
                add_backup: add_backup.clone(),
            }
        ));

        // This is a weird way of giving a way for method to make new clones
        // of the Rc.
        myself.borrow_mut().myself.replace(myself.clone());

        // Include.
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
        vbox.set_widget_name("include_box");
        stack.add_titled(&vbox, Some("include"), "Folders to include");

        let include_top = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        vbox.append(&include_top);

        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
        label.set_markup("<b>Paths to backup</b>");
        include_top.append(&label);

        let add_include_button = gtk::Button::new();
        add_include_button.set_halign(gtk::Align::End);
        add_include_button.set_hexpand(true);
        add_include_button.set_icon_name("list-add-symbolic");
        include_top.append(&add_include_button);

        let include_list = gtk::ListBox::new();
        include_list.set_widget_name("edit_include_list");
        include_list.set_selection_mode(gtk::SelectionMode::None);
        include_list.set_show_separators(true);
        include_list.set_css_classes(&["rich-list"]);
        vbox.append(&include_list);

        // Exclude.
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
        vbox.set_widget_name("exclude_box");
        stack.add_titled(&vbox, Some("exclude"), "Patterns to exclude");

        let exclude_top = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        vbox.append(&exclude_top);

        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
        label.set_markup("<b>Patterns to exclude</b>");
        exclude_top.append(&label);

        let add_exclude_button = gtk::Button::new();
        add_exclude_button.set_halign(gtk::Align::End);
        add_exclude_button.set_hexpand(true);
        add_exclude_button.set_icon_name("list-add-symbolic");
        exclude_top.append(&add_exclude_button);

        let exclude_list = gtk::ListBox::new();
        exclude_list.set_widget_name("edit_exclude_list");
        exclude_list.set_selection_mode(gtk::SelectionMode::None);
        exclude_list.set_show_separators(true);
        exclude_list.set_css_classes(&["rich-list"]);
        vbox.append(&exclude_list);

        // Create a local mutable borrow of self so we can properly
        // add default entries to include/exclude and connect signals.
        let mut edit_ui = myself.borrow_mut();

        // FIXME: this is wrong, we need to ask the server what UID it is
        // running with...
        let default_include_path = match users::get_effective_uid() {
            0 => std::path::PathBuf::from("/"),
            _ => dirs::home_dir().unwrap(),
        };

        let default_row = edit_ui.new_include_row(default_include_path);
        include_list.append(&default_row);

        for row in edit_ui.default_exclude_patterns() {
            exclude_list.append(&row);
        }

        let include_self = myself.clone();
        add_include_button.connect_clicked(
            clone!(@weak include_list => move |_| {
                let parent_window = include_self.borrow().window.clone();
                let file_picker = gtk::FileChooserDialog::new(
                    Some("Choose folder to include..."),
                    Some(&parent_window),
                    gtk::FileChooserAction::SelectFolder,
                    &[
                        ("Cancel", gtk::ResponseType::Cancel),
                        ("Accept", gtk::ResponseType::Accept),
                    ]
                );
                file_picker.set_modal(true);

                let picker_self = include_self.clone();
                file_picker.run_async(move |dialog, response| {
                    dialog.close();

                    if response == gtk::ResponseType::Accept {
                        let path = dialog.file().unwrap().path().unwrap();
                        let row  = picker_self.borrow_mut()
                            .new_include_row(path);
                        include_list.append(&row);
                    }
                });
            })
        );

        let exclude_self = myself.clone();
        add_exclude_button.connect_clicked(
            clone!(@weak exclude_list => move |_| {
                let row  = exclude_self.borrow_mut()
                     .new_exclude_row("");
                 exclude_list.append(&row);
            })
        );

        let back_self = myself.clone();
        back_button.connect_clicked(move |_| {
            let myself = back_self.borrow();
            myself.go(Go::Back);
        });

        let forward_self = myself.clone();
        forward_button.connect_clicked(move |_| {
            let myself = forward_self.borrow();
            myself.go(Go::Forward);
        });

        let add_self = myself.clone();
        add_backup.connect_clicked(
            clone!(@weak name_entry, @weak type_combo, @weak identifier, @weak path,
                @weak key_entry, @weak secret_entry,
                @weak password => move |_| {
                let repo_type = type_combo.active_id()
                    .expect("Combo box should never be empty")
                    .to_string();
                let identifier = identifier.text().to_string();
                let path = path.text().to_string();
                let password = password.text().to_string();

                let repository_str = format!("{}:{}:{}", repo_type, identifier, path);
                let repository = Repository::from(repository_str.as_str());

                let mut key_id: Option<String> = None;
                let mut key_secret: Option<String> = None;
                // This could be written as an if let, but we will add more cases
                // here, so we make it a match.
                match repository.kind {
                    RepositoryKind::B2 => {
                        key_id.replace(key_entry.text().to_string());
                        key_secret.replace(secret_entry.text().to_string());
                    }
                    _ => ()
                }

                let include = add_self.borrow()
                    .include.iter()
                    .map(|entry| PathBuf::from(entry.text().to_string()))
                    .collect();

                let exclude = add_self.borrow()
                    .exclude.iter()
                    .map(|entry| entry.text().to_string())
                    .collect();

                let backup = Backup {
                    name: name_entry.text().to_string(),
                    repository,
                    password,
                    key_id,
                    key_secret,
                    include,
                    exclude,
                };

                let myself = add_self.clone();
                let app = application.clone();
                MainContext::default().spawn_local(async move {
                    let connection = match Server::connect(app.clone()).await {
                        Ok(c) => c,
                        Err(_) => return,
                    };

                    if let Err(error) = connection.send_message(
                        ClientMessage::CreateBackup(ClientMessageCreateBackup {backup})
                    ).await { println!("Error creating backup: {:#?}", error) }

                    match connection.read_response().await {
                        Ok(response) => {
                            dbg!(&response);
                            if let Some(error) = response.error {
                                let error = match error {
                                    ServerError::Configuration(e) |
                                    ServerError::RepoInit(e) => e,
                                };
                                let parent_window = myself.borrow().window.clone();
                                let dialog = gtk::MessageDialogBuilder::new()
                                    .transient_for(&parent_window)
                                    .modal(true)
                                    .message_type(gtk::MessageType::Error)
                                    .buttons(gtk::ButtonsType::Close)
                                    .text("Failed to create backup.")
                                    .secondary_text(&error)
                                    .build();
                                dialog.run_future().await;
                                dialog.close();
                            } else {
                                myself.borrow_mut().clear();
                                app.borrow_mut().update();
                            }
                        },
                        Err(error) => {
                            dbg!(error);
                        },
                    };
                });
            })
        );

        // Disable or enable add_backup based on various inputs.
        let entries = vec![
            edit_ui.name.clone().upcast::<gtk::Editable>(),
            edit_ui.path.clone().upcast::<gtk::Editable>(),
            edit_ui.password.clone().upcast::<gtk::Editable>(),
            edit_ui.confirm.clone().upcast::<gtk::Editable>(),
        ];

        for entry in entries {
            let edit_ui = myself.clone();
            entry.connect_changed(move |_| {
                // If we are clearing the form after a backup was added, this handler
                // will be triggered, but the edit ui will be borrowed mutably by the
                // clear method. We don't need to care about state, as it will be reset.
                match edit_ui.try_borrow_mut() {
                    Ok(mut edit_ui) => edit_ui.update_state(),
                    Err(_) => (),
                }
            });
        }

        // Explicitly drop the borrow so we can move self out.
        drop(edit_ui);

        myself
    }

    fn go(&self, direction: Go) {
        let current_page = self.stack.visible_child_name().unwrap().to_string();
        if let Go::Back = direction {
            match current_page.as_str() {
                "repository" => unreachable!(),
                "include" => {
                    self.stack.set_visible_child_name("repository");
                    self.back_button.set_sensitive(false);
                },
                "exclude" => {
                    self.stack.set_visible_child_name("include");
                    self.forward_button.set_visible(true);
                    self.add_backup.set_visible(false);
                },
                _ => unreachable!(),
            }
         } else {
            match current_page.as_str() {
                "repository" => {
                    self.stack.set_visible_child_name("include");
                    self.back_button.set_sensitive(true);
                },
                "include" => {
                    self.stack.set_visible_child_name("exclude");
                    self.forward_button.set_visible(false);
                    self.add_backup.set_visible(true);
                },
                "exclude" => unreachable!(),
                _ => unreachable!(),
            }
        }
    }

    pub fn open(&self) {
        self.window.present();
    }

    fn default_exclude_patterns(&mut self) -> Vec<gtk::ListBoxRow> {
        let mut patterns = Vec::<String>::new();
        if cfg!(target_os = "macos") {
            let home_dir = dirs::home_dir()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // FIXME: we need to ask the server if it running in privileged mode to
            // filter which patterns to use as default.
            &[
                ".cache",
                "Caches",
                &format!("{}/.Trash", home_dir),
                &format!("{}/Library", home_dir),
                "/private/var/folders",
                "/private/var/networkd/db",
                "/private/var/protected/trustd/private",
                "/private/var/db"
            ].iter().for_each(|p| patterns.push(p.to_string()));
        }

        if cfg!(target_os = "linux") {
            patterns.push(".cache".to_string());
            patterns.push("/var/lib/systemd/coredump".to_string());
        }

        patterns
            .iter()
            .map(|p| self.new_exclude_row(&p))
            .collect()
    }

    fn new_exclude_row(&mut self, initial_text: &str) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRowBuilder::new()
            .activatable(false)
            .selectable(false)
            .build();

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        row.set_child(Some(&hbox));

        let entry = gtk::Entry::new();
        entry.set_text(initial_text);
        hbox.append(&entry);

        let button = gtk::Button::new();
        button.set_hexpand(true);
        button.set_halign(gtk::Align::End);
        button.set_icon_name("list-remove-symbolic");
        hbox.append(&button);

        let myself = self.clone_self();
        button.connect_clicked(
            clone!(@weak row, @weak entry => move |_| {
                // Remove entry from our list of include entries.
                let entry = entry.upcast::<gtk::Editable>();
                myself.borrow_mut()
                    .exclude
                    .retain(|x| x != &entry);

                // Unparent row, which should destroy everything.
                let parent = row.parent().unwrap()
                    .downcast::<gtk::ListBox>().unwrap();
                parent.remove(&row);
            })
        );

        self.exclude.push(entry.clone().upcast::<gtk::Editable>());

        // Schedule grabbing focus as trying to grab it now won't work.
        glib::source::idle_add_local(move || {
            entry.grab_focus();
            glib::Continue(false)
        });

        row
     }

    fn new_include_row(&mut self, path: PathBuf) -> gtk::ListBoxRow {
        let path_string = path.to_string_lossy().to_string();

        let row = gtk::ListBoxRowBuilder::new()
            .activatable(false)
            .selectable(false)
            .build();

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        row.set_child(Some(&hbox));

        let label = gtk::Label::new(Some(&path_string));
        label.set_halign(gtk::Align::Fill);
        hbox.append(&label);

        let entry = gtk::Entry::new();
        entry.set_visible(false);
        hbox.append(&entry);

        entry.set_text(&path_string);

        let button = gtk::Button::new();
        button.set_hexpand(true);
        button.set_halign(gtk::Align::End);
        button.set_icon_name("list-remove-symbolic");
        hbox.append(&button);

        let myself = self.clone_self();
        button.connect_clicked(
            clone!(@weak row, @weak entry => move |_| {
                // Remove entry from our list of include entries.
                let entry = entry.upcast::<gtk::Editable>();
                myself.borrow_mut()
                    .include
                    .retain(|x| x != &entry);

                // Unparent row, which should destroy everything.
                let parent = row.parent().unwrap()
                    .downcast::<gtk::ListBox>().unwrap();
                parent.remove(&row);
            })
        );

        self.include.push(entry.upcast::<gtk::Editable>());

        row
    }

    fn clone_self(&self) -> Rc<RefCell<Self>> {
        self.myself.as_ref().unwrap().clone()
    }

    fn update_state(&mut self) {
        // Start optimistic, see if anything makes us want to disable the button.
        let mut sensitive = true;

        if self.name.text().to_string().trim().is_empty() {
            sensitive = false;
        }

        let password = self.password.text().to_string();
        let confirm = self.confirm.text().to_string();
        if password.is_empty() || password != confirm {
            sensitive = false;
        }

        self.forward_button.set_sensitive(sensitive);
        self.add_backup.set_sensitive(sensitive);
    }

    fn clear(&mut self) {
        self.window.hide();

        self.name.set_text("");
        self.kind.set_active_id(Some("Local"));
        self.identifier.set_text("");
        self.key_id.set_text("");
        self.key_secret.set_text("");
        self.path.set_text("");
        self.password.set_text("");
        self.confirm.set_text("");
        self.include.iter_mut()
            .for_each(|entry| entry.set_text(""));
        self.add_backup.set_sensitive(false);
    }
}

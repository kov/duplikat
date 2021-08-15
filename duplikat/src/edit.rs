use std::{cell::RefCell, rc::Rc};
use std::str::FromStr;
use std::path::PathBuf;
use glib::{MainContext, clone};
use gtk::prelude::*;
use duplikat_types::*;
use strum::IntoEnumIterator;
use crate::{Application, StackPage};
use crate::server::Server;

pub struct CreateEditUI {
    pub container: gtk::Box,
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
    add_backup: gtk::Button,
}

fn next_row_num(num: &mut i32) -> i32 {
    *num += 1;
    *num
}

impl CreateEditUI {
    pub(crate) fn new(application: Rc<RefCell<Application>>) -> Rc<RefCell<Self>> {
        let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        main_vbox.append(&hbox);

        let grid = gtk::Grid::new();
        hbox.append(&grid);

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


        // Add backup button, and signal handlers.
        let add_backup = gtk::Button::with_label("Add Backup");
        add_backup.set_widget_name("add_backup_button");
        add_backup.set_halign(gtk::Align::End);
        add_backup.set_css_classes(&["suggested-action"]);
        add_backup.set_sensitive(false);
        main_vbox.append(&add_backup);

        let myself = Rc::new(RefCell::new(
            CreateEditUI {
                container: main_vbox.clone(),
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
                add_backup: add_backup.clone(),
            }
        ));

        // This is a weird way of giving a way for method to make new clones
        // of the Rc.
        myself.borrow_mut().myself.replace(myself.clone());

        // Include / exclude lists.
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
        vbox.set_widget_name("include_exclude_box");
        hbox.append(&vbox);

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

        // Create a local mutable borrow of self so we can properly
        // add default entries to include/exclude and connect signals.
        let mut edit_ui = myself.borrow_mut();

        let default_include_path = match users::get_effective_uid() {
            0 => std::path::PathBuf::from("/"),
            _ => dirs::home_dir().unwrap(),
        };

        let default_row = edit_ui.new_include_row(default_include_path);
        include_list.append(&default_row);

        let include_app = application.clone();
        let include_self = myself.clone();
        add_include_button.connect_clicked(
            clone!(@weak include_list => move |_| {
                let main_window = include_app.borrow().main_window.clone();
                let file_picker = gtk::FileChooserDialog::new(
                    Some("Choose folder to include..."),
                    Some(&main_window),
                    gtk::FileChooserAction::SelectFolder,
                    &[
                        ("Cancel", gtk::ResponseType::Cancel),
                        ("Accept", gtk::ResponseType::Accept),
                    ]
                );

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

                let backup = Backup {
                    name: name_entry.text().to_string(),
                    repository,
                    password,
                    key_id,
                    key_secret,
                    include,
                    exclude: vec![".cache".to_string()],
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
                                let main_window = app.borrow().main_window.clone();
                                let dialog = gtk::MessageDialogBuilder::new()
                                    .transient_for(&main_window)
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
                                app.borrow_mut().set_stack_page(StackPage::Overview);
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
        let mut add_backup_sensitive = true;

        if self.name.text().to_string().trim().is_empty() {
            add_backup_sensitive = false;
        }

        let password = self.password.text().to_string();
        let confirm = self.confirm.text().to_string();
        if password.is_empty() || password != confirm {
            add_backup_sensitive = false;
        }

        self.add_backup.set_sensitive(add_backup_sensitive);
    }

    fn clear(&mut self) {
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

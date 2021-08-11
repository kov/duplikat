use std::{cell::RefCell, rc::Rc};
use std::str::FromStr;
use std::path::PathBuf;
use glib::{MainContext, clone};
use gtk::prelude::*;
use duplikat_types::*;
use strum::IntoEnumIterator;
use crate::server::Server;

pub struct CreateEditUI {
    pub container: gtk::Box,
}

fn next_row_num(num: &mut i32) -> i32 {
    *num += 1;
    *num
}

impl CreateEditUI {
    pub(crate) fn new() -> Rc<RefCell<Self>> {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);

        let grid = gtk::Grid::new();
        hbox.append(&grid);

        let mut row_num = -1i32;

        // Name
        let label = gtk::Label::new(Some("Name"));
        grid.attach(&label, next_row_num(&mut row_num), 0, 1, 1);

        let name_entry = gtk::Entry::new();
        name_entry.set_text("kov");
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
        path.set_text("/home/kov/.config/duplikatd/storage");

        grid.attach_next_to(&path, Some(&label), gtk::PositionType::Right, 1, 1);

        // Password
        let label = gtk::Label::new(Some("Password"));
        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let password = gtk::PasswordEntry::new();
        password.set_text("lala");
        password.set_show_peek_icon(true);

        grid.attach_next_to(&password, Some(&label), gtk::PositionType::Right, 1, 1);

        // Password
        let label = gtk::Label::new(Some("Confirm Password"));
        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

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
            clone!(@weak name_entry, @weak key_entry, @weak secret_entry => move |_| {
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

                let backup = Backup {
                    name: name_entry.text().to_string(),
                    repository,
                    password,
                    key_id,
                    key_secret,
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

        grid.attach(&add_backup, 1, next_row_num(&mut row_num), 1, 1);

        Rc::new(RefCell::new(
            CreateEditUI {
                container: hbox.clone(),
            }
        ))
    }
}

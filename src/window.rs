use isahc::prelude::*;
use duplikat_types::*;
use glib::{Cast, ObjectExt, clone};
use gtk::prelude::WidgetExt;
use libadwaita::prelude::ComboRowExt;
use std::rc::Rc;
use crate::widgets::BackupRow;

async fn get_backups_list(list_box: gtk::ListBox) {
    let mut res = Request::get("http://localhost:7667/backups")
        .body(())
        .unwrap()
        .send_async()
        .await;
    match &mut res {
        Ok(res) => {
            let backups = res.json::<Vec<Backup>>().unwrap();
            println!("{:#?}", res);
            println!("{:#?}", backups);
            for b in backups {
                let row = BackupRow::new(&b);
                list_box.append(&row);
            }
        },
        Err(e) => println!("Error: {:#?}", e),
    }
}

pub enum WindowViews {
    List,
    Create,
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    pub builder: gtk::Builder,
}

impl Window {
    pub fn new() -> Rc<Self> {
        let builder = gtk::Builder::new();
        builder.add_from_resource("/br/dev/kov/Duplikat/window.ui")
            .expect("Failed to load ui file.");
        let widget: gtk::ApplicationWindow = builder
            .object("window")
            .expect("Failed to find the window object");

        get_widget!(builder, gtk::ListBox, list_box);
        let list_box = list_box.clone();
        glib::MainContext::default().spawn_local(
            get_backups_list(list_box)
        );

        get_widget!(builder, libadwaita::ComboRow, backup_destination_type);
        Window::setup_destination_type_signals(builder.clone(), backup_destination_type);

        let myself = Rc::new(Self { widget, builder });

        get_widget!(myself.builder, gtk::Stack, main_view);
        main_view.connect_visible_child_notify(
            clone!(@weak myself => move |_| {
                myself.update_state();
            }),
        );

        myself.update_state();

        myself
    }

    pub fn set_view(&self, view: WindowViews) {
        get_widget!(self.builder, gtk::Stack, main_view);

        let child_name = match view {
            WindowViews::List => "list",
            WindowViews::Create => "create",
        };

        main_view.set_visible_child_name(child_name);
    }

    fn setup_destination_type_signals(builder: gtk::Builder, destination_type: libadwaita::ComboRow) {
        destination_type.connect_property_selected_item_notify(
            clone!(@weak destination_type => move |_| {
                get_widget!(builder, gtk::Widget, backup_local_path);
                backup_local_path.hide();

                get_widget!(builder, gtk::Widget, backup_sftp_host);
                backup_sftp_host.hide();

                get_widget!(builder, gtk::Widget, backup_b2_bucket);
                backup_b2_bucket.hide();

                let item_name = destination_type.selected_item()
                    .unwrap() // there must be an item
                    .downcast::<gtk::StringObject>()
                    .unwrap() // there must be a valid string
                    .string()
                    .to_string();

                match RepositoryKind::from(item_name.as_str()) {
                    RepositoryKind::Local => backup_local_path.show(),
                    RepositoryKind::SFTP => backup_sftp_host.show(),
                    RepositoryKind::B2 => backup_b2_bucket.show(),
                }
            })
        );
    }

    fn update_state(&self) {
        get_widget!(self.builder, gtk::Stack, main_view);

        let visible_child_name = main_view
            .visible_child_name()
            .map(|s| s.to_string())
            .unwrap_or("none".to_string());

        get_widget!(self.builder, gtk::Widget, win_add);
        get_widget!(self.builder, gtk::Widget, win_go_previous);

        match visible_child_name.as_str() {
            "list" => {
                win_add.show();
                win_go_previous.hide();
            },
            "create" => {
                win_add.hide();
                win_go_previous.show();

                self.create_backup();
            },
            _ => {
                unimplemented!();
            }
        }
    }

    fn create_backup(&self) {
        // Simple test
        let backup = Backup {
            name: "uva".to_string(),
            repository: Repository {
                kind: RepositoryKind::B2,
                identifier: "fedora-vm-uva".to_string(),
                path: "/system".to_string(),
            },
            password: "pass".to_string()
        };

        let client = reqwest::blocking::Client::new();
        let res = client.post("http://localhost:7667/backups")
            .body(serde_json::to_string(&backup).unwrap())
            .send().unwrap();
        println!("{:#?}", res);
        println!("{}", res.text().unwrap());
    }
}

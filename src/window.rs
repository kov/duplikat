use isahc::prelude::*;
use duplikat_types::*;
use glib::{Cast, clone};
use gtk::prelude::{
    ButtonExt, ComboBoxExt, EditableExt, FileChooserExt,
    FileExt, NativeDialogExt, WidgetExt
};
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

        Window::setup_signals(builder.clone());

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

    fn setup_signals(builder: gtk::Builder) {
        // Destination combo box which controls which other widgets are shown
        // or hidden.
        get_widget!(builder, gtk::ComboBox, backup_destination_type);
        let cbuilder = builder.clone();
        backup_destination_type.connect_changed(
            clone!(@weak backup_destination_type => move |_| {
                let backup_destination_type = backup_destination_type.downcast::<gtk::ComboBoxText>().unwrap();
                let item_name = backup_destination_type.active_text()
                    .unwrap() // there must be a valid string
                    .to_string();

                let name_prefix_to_show = match RepositoryKind::from(item_name.as_str()) {
                    RepositoryKind::Local => "backup_local",
                    RepositoryKind::SFTP => "backup_sftp",
                    RepositoryKind::B2 => "backup_b2_bucket",
                };

                let conditional_widget_names = vec![
                    "backup_local_path",
                    "backup_local_path_button",
                    "backup_sftp_host",
                    "backup_sftp_host_entry",
                    "backup_b2_bucket",
                    "backup_b2_bucket_entry"
                ];

                for name in conditional_widget_names {
                    let w = cbuilder.object::<gtk::Widget>(name)
                        .expect(format!("Could not find {}", name).as_str());

                    if name.starts_with(name_prefix_to_show) {
                        w.show();
                    } else {
                        w.hide();
                    }
                };
            })
        );

        // Button to select a folder in case backup_local is used. Uses
        // FileChooserNative for better cross-platform integration.
        get_widget!(builder, gtk::Entry, hidden_backup_local_path);
        let mut default_path = glib::home_dir().unwrap();
        default_path.push("Backups");
        hidden_backup_local_path.set_text(
            &default_path.to_string_lossy()
        );

        get_widget!(builder, gtk::Button, backup_local_path_button);
        backup_local_path_button.connect_clicked(
            clone!(@weak backup_local_path_button => move |_| {
                get_widget!(builder, gtk::Window, window);

                let chooser = gtk::FileChooserNative::new(
                    Some("Choose backup folder"),
                    Some(&window),
                    gtk::FileChooserAction::SelectFolder,
                    None,
                    None
                );
                chooser.show();

                let main_loop = glib::MainLoop::new(None, true);

                let cmain_loop = main_loop.clone();
                chooser.connect_response(
                    clone!(@weak builder, @weak chooser => move |_, response_type| {
                        cmain_loop.quit();

                        if response_type != gtk::ResponseType::Accept {
                            return;
                        }

                        let gfile = chooser.file().unwrap();
                        let info = gfile.query_info(
                            &gio::FILE_ATTRIBUTE_STANDARD_DISPLAY_NAME,
                            gio::FileQueryInfoFlags::NONE,
                            gio::NONE_CANCELLABLE
                        ).unwrap();

                        get_widget!(builder, gtk::Label, backup_local_path_label);
                        backup_local_path_label.set_label(&info.display_name());

                        get_widget!(builder, gtk::Entry, hidden_backup_local_path);
                        hidden_backup_local_path.set_text(
                            &gfile.path().unwrap().to_string_lossy()
                        );
                    })
                );
                main_loop.run();
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
            },
            _ => {
                unimplemented!();
            }
        }
    }

    pub fn create_backup(&self) {
        get_widget!(self.builder, gtk::Entry, backup_name_entry);
        let name = backup_name_entry.text().to_string();

        get_widget!(self.builder, gtk::ComboBoxText, backup_destination_type);
        let type_name = backup_destination_type.active_text()
            .unwrap() // there must be a valid string
            .to_string();
        let kind = RepositoryKind::from(type_name.as_str());

        let identifier = "".to_string();

        let path = match kind {
            RepositoryKind::Local => {
                get_widget!(self.builder, gtk::Entry, hidden_backup_local_path);
                hidden_backup_local_path.text().to_string()
            },
            _ => unimplemented!(),
        };

        // FIXME: error handling.
        std::fs::create_dir_all(&path).unwrap();

        get_widget!(self.builder, gtk::Entry, backup_password_entry);
        let password = backup_password_entry.text().to_string();

        let backup = Backup {
            name,
            repository: Repository {
                kind,
                identifier,
                path,
            },
            password,
        };

        let client = reqwest::blocking::Client::new();
        let res = client.post("http://localhost:7667/backups")
            .body(serde_json::to_string(&backup).unwrap())
            .send().unwrap();
        println!("{:#?}", res);
        println!("{}", res.text().unwrap());
    }
}

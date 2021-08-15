use std::{cell::RefCell, collections::HashMap, rc::Rc};
use duplikat_types::*;
use glib::{MainContext, clone};
use gtk::prelude::*;
use crate::Application;
use crate::server::Server;

pub struct OverviewUI {
    pub container: gtk::ListBox,
    rows: HashMap<String, BackupRow>,
    application: Rc<RefCell<Application>>,
}

struct BackupRow {
    bytes: gtk::Label,
}

fn to_human_readable(bytes: u64) -> String {
    let tiers = vec!["KiB", "MiB", "GiB", "TiB"];
    let mut bytes = bytes as f64;
    for tier in tiers {
        bytes /= 1024f64;
        if bytes < 1000f64 {
            return format!("{:.2} {}", bytes, tier);
        }
    }
    "NaN".to_string()
}

impl OverviewUI {
    pub(crate) fn new(application: Rc<RefCell<Application>>) -> Rc<RefCell<Self>> {
        let listbox = gtk::ListBox::new();
        listbox.set_widget_name("overview_listbox");
        listbox.set_selection_mode(gtk::SelectionMode::None);
        listbox.set_show_separators(true);
        listbox.set_css_classes(&["rich-list"]);

        let overview = Rc::new(RefCell::new(OverviewUI {
            container: listbox.clone(),
            rows: Default::default(),
            application: application.clone(),
        }));

        let o = overview.clone();
        let a = application.clone();
        clone!(@weak listbox => move || {
            MainContext::default().spawn_local(
                async move {
                    let mut overview = o.borrow_mut();
                    let connection = match Server::connect(a).await {
                        Ok(c) => c,
                        Err(_) => return,
                    };

                    if let Err(error) = connection.send_message(ClientMessage::ListBackups).await {
                        println!("Error listing backups: {:#?}", error);
                    };

                    while let Ok(message) = connection.read_message().await {
                        if let Some(message) = message {
                            dbg!(&message);
                            match message {
                                ResticMessage::BackupsList(backups) => {
                                    for name in backups.list {
                                        let row = overview.create_row_for_name(&name);
                                        listbox.append(&row);
                                    }
                                },
                                ResticMessage::BackupStats(stats) => {
                                    dbg!(&stats);
                                    let row = overview.rows.get(&stats.name).unwrap();
                                    row.bytes.set_text(&to_human_readable(stats.total_size));
                                },
                                _ => unimplemented!(),
                            }
                        } else {
                            break;
                        }
                    }
                }
            );
        })();

        overview
    }

    fn create_row_for_name(&mut self, name: &str) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        row.set_child(Some(&hbox));

        let label = gtk::Label::new(Some(name));
        hbox.append(&label);

        let run_button = gtk::Button::with_label("Run");
        hbox.append(&run_button);

        let progress_bar = gtk::ProgressBar::new();
        progress_bar.set_valign(gtk::Align::Center);
        progress_bar.set_halign(gtk::Align::Fill);
        hbox.append(&progress_bar);

        // Make an owned instance so that it can be moved into the closure.
        let application = self.application.clone();
        let backup_name = name.to_string();
        run_button.connect_clicked(
            clone!(@weak progress_bar => move |_| {
                let application = application.clone();
                let name = backup_name.clone();
                MainContext::default().spawn_local(async move {
                    let run_backup_message = ClientMessage::RunBackup(
                        ClientMessageRunBackup {
                            name,
                        }
                    );

                    let connection = match Server::connect(application).await {
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

        let bytes_label = gtk::Label::new(None);
        hbox.append(&bytes_label);

        // Add this row to our map, so we can easily access it when updating data
        // for a backup.
        self.rows.insert(
            name.to_string(),
            BackupRow {
                bytes: bytes_label,
            }
        );

        row
    }
}

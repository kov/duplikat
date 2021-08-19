use std::{cell::RefCell, collections::HashMap, rc::Rc};
use duplikat_types::*;
use glib::{MainContext, clone};
use gtk::prelude::*;
use crate::Application;
use crate::server::Server;
use crate::utils::next_row_num;

pub struct OverviewUI {
    pub container: gtk::ListBox,
    rows: HashMap<String, BackupRow>,
    application: Rc<RefCell<Application>>,
}

struct BackupRow {
    bytes: gtk::Label,
    files: gtk::Label,
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

fn seconds_to_human_readable(seconds: u64) -> String {
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if seconds < 60 {
        if seconds > 1 {
            format!("{} seconds", seconds)
        } else {
            format!("{} second", seconds)
        }
    } else if minutes < 60 {
        if minutes > 1 {
            format!("{} minutes", minutes)
        } else {
            format!("{} minute", minutes)
        }
    } else if hours < 24 {
        if hours > 1 {
            format!("{} hours", hours)
        } else {
            format!("{} hour", hours)
        }
    } else {
        if days > 1 {
            format!("{} days", days)
        } else {
            format!("{} day", days)
        }
    }
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

        overview.borrow().update();

        overview
    }

    pub fn update(&self) {
        let a = self.application.clone();
        MainContext::default().spawn_local(
            async move {
                let application = a.clone();
                let overview = application.borrow_mut().overview.as_mut().unwrap().clone();
                let connection = match Server::connect(a.clone()).await {
                    Ok(c) => c,
                    Err(_) => return,
                };

                if let Err(error) = connection.send_message(ClientMessage::ListBackups).await {
                    println!("Error listing backups: {:#?}", error);
                };

                let listbox = overview.borrow().container.clone();
                while let Some(row) = listbox.row_at_index(0) {
                    listbox.remove(&row);
                }

                while let Ok(message) = connection.read_message().await {
                    if let Some(message) = message {
                        dbg!(&message);
                        match message {
                            ResticMessage::BackupsList(backups) => {
                                for backup in backups.list {
                                    let row = overview.borrow_mut().create_row_for_backup(&backup).clone();
                                    listbox.append(&row);
                                }
                            },
                            ResticMessage::BackupStats(stats) => {
                                dbg!(&stats);
                                let overview = overview.borrow_mut();
                                let row = overview.rows.get(&stats.name).unwrap();
                                row.bytes.set_markup(
                                    &to_human_readable(stats.total_size)
                                );
                                row.files.set_markup(
                                    &format!("{}", stats.total_file_count)
                                );
                            },
                            _ => unimplemented!(),
                        }
                    } else {
                        break;
                    }
                }
            }
        );
    }

    fn create_row_for_backup(&mut self, backup: &Backup) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        let frame = gtk::Frame::new(Some(&backup.name));
        row.set_child(Some(&frame));

        let grid = gtk::Grid::new();
        frame.set_child(Some(&grid));

        let mut type_text = String::new();
        match backup.repository.kind {
            RepositoryKind::Local => {
                type_text.push_str(
                    &format!("<b>Local</b> ({})",
                        &backup.repository.path
                    )
                );
            },
            RepositoryKind::B2 => {
                type_text.push_str(
                    &format!("<b>Backblaze B2</b> ({}{})",
                        &backup.repository.identifier,
                        &backup.repository.path
                    )
                );
            },
            RepositoryKind::SFTP => {
                type_text.push_str(
                    &format!("<b>SFTP</b> ({}{})",
                        &backup.repository.identifier,
                        &backup.repository.path
                    )
                );
            },
        }

        let mut row_num = -1i32;

        let label = gtk::Label::new(None);
        label.set_markup(&type_text);

        grid.attach(&label, 0, next_row_num(&mut row_num), 2, 1);

        // This should be a logo for the type of repository.
        let logo = gtk::Image::from_icon_name(Some("image-x-generic"));
        logo.set_halign(gtk::Align::End);
        logo.set_hexpand(true);

        grid.attach_next_to(&logo, Some(&label), gtk::PositionType::Right, 1, 1);

        let label = gtk::Label::new(None);
        label.set_markup("<b>Total size:</b>");

        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let bytes_label = gtk::Label::new(Some("calculating..."));

        grid.attach_next_to(&bytes_label, Some(&label), gtk::PositionType::Right, 1, 1);

        let label = gtk::Label::new(None);
        label.set_markup("<b>File count:</b>");

        grid.attach(&label, 0, next_row_num(&mut row_num), 1, 1);

        let files_label = gtk::Label::new(Some("calculating..."));

        grid.attach_next_to(&files_label, Some(&label), gtk::PositionType::Right, 1, 1);

        let progress_bar = gtk::ProgressBar::new();
        progress_bar.set_show_text(true);
        progress_bar.set_halign(gtk::Align::Fill);
        progress_bar.set_hexpand(true);
        progress_bar.set_visible(false);

        grid.attach(&progress_bar, 0, next_row_num(&mut row_num), 2, 1);

        let run_button = gtk::Button::with_label("Backup now");
        run_button.set_halign(gtk::Align::End);
        run_button.set_hexpand(true);
        run_button.set_css_classes(&["suggested-action"]);

        grid.attach_next_to(&run_button, Some(&progress_bar), gtk::PositionType::Right, 1, 1);

        let cancel_button = gtk::Button::with_label("Cancel");
        cancel_button.set_visible(false);
        cancel_button.set_halign(gtk::Align::End);
        cancel_button.set_hexpand(true);

        grid.attach_next_to(&cancel_button, Some(&progress_bar), gtk::PositionType::Right, 1, 1);

        // Make an owned instance so that it can be moved into the closure.
        let application = self.application.clone();
        let backup_name = backup.name.clone();
        run_button.connect_clicked(
            clone!(@weak progress_bar, @weak cancel_button => move |button| {
                let application = application.clone();
                let name = backup_name.clone();
                let button = button.clone();
                MainContext::default().spawn_local(async move {
                    let run_backup_message = ClientMessage::RunBackup(
                        ClientMessageRunBackup {
                            name,
                        }
                    );

                    let connection = match Server::connect(application.clone()).await {
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
                                    button.set_visible(false);
                                    cancel_button.set_visible(true);
                                    progress_bar.set_visible(true);
                                    progress_bar.set_fraction(status.percent_done);
                                    if let Some(seconds) = status.seconds_remaining {
                                        let time_str = seconds_to_human_readable(seconds);
                                        progress_bar.set_text(Some(
                                            &format!("{}% ({} left)",
                                                (status.percent_done * 100f64) as u64,
                                                time_str
                                            )
                                        ));
                                    }
                                },
                                ResticMessage::Summary(_) => {
                                    button.set_visible(true);
                                    cancel_button.set_visible(false);
                                    progress_bar.set_visible(false);
                                    progress_bar.set_fraction(0.);
                                    application.borrow().overview.as_ref().unwrap().borrow().update();
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

        // Add this row to our map, so we can easily access it when updating data
        // for a backup.
        self.rows.insert(
            backup.name.clone(),
            BackupRow {
                bytes: bytes_label,
                files: files_label,
            }
        );

        row
    }
}

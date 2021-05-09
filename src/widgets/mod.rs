use glib::ToValue;
use gtk::glib;
use duplikat_types::*;

mod backup_row;

glib::wrapper! {
    pub struct BackupRow(ObjectSubclass<backup_row::BackupRow>)
        @extends gtk::Widget;
}

impl BackupRow {
    pub fn new(backup: &Backup) -> Self {
        let mut properties: Vec<(&str, &dyn ToValue)> = vec![];
        properties.push(("name", &backup.name));
        glib::Object::new(&properties).expect("Failed to create BackupRow")
    }
}

use glib::{BindingFlags, ParamFlags, ParamSpec, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita::subclass::prelude::*;

use once_cell::sync::Lazy;
use std::cell::RefCell;

#[derive(Default)]
pub struct BackupRow {
    name: RefCell<Option<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for BackupRow {
    const NAME: &'static str = "BackupRow";
    type Type = super::BackupRow;
    type ParentType = libadwaita::ExpanderRow;
}

impl ObjectImpl for BackupRow {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![ParamSpec::new_string(
                "name",
                "name",
                "name",
                None,
                ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "name" => {
                self.name.replace(value.get().unwrap());
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "name" => self.name.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.bind_property("name", obj, "subtitle")
            .flags(BindingFlags::BIDIRECTIONAL | BindingFlags::SYNC_CREATE)
            .build();
    }
}

impl WidgetImpl for BackupRow {}

impl ListBoxRowImpl for BackupRow {}

impl PreferencesRowImpl for BackupRow {}

impl ExpanderRowImpl for BackupRow {}
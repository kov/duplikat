use isahc::prelude::*;
use duplikat_types::*;
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

pub struct Window {
    pub widget: gtk::ApplicationWindow,
}

impl Window {
    pub fn new() -> Self {
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

        Self { widget }
    }
}

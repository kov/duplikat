use gtk::prelude::*;
use gtk::ApplicationWindow;
use duplikat_types::*;

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let app = gtk::Application::new(Some("br.dev.kov.Duplikat"), Default::default());
    app.connect_activate(move |app| {
        let window = create_ui(app);
        window.present();
    });

    let ret = app.run();
    std::process::exit(ret);
}

fn create_ui(app: &gtk::Application) -> ApplicationWindow {
    let window = gtk::ApplicationWindow::new(app);
    window.set_title(Some("Duplikat"));

    let listbox = gtk::ListBoxBuilder::new()
        .css_classes(vec!["rich-list".to_string()])
        .build();

    window.set_child(Some(&listbox));

    let add_backup = gtk::Button::with_label("Add Backup");
    add_backup.connect_clicked(move |_| {
        let backup = Backup {
            name: "local".to_string(),
            repository: Repository {
                kind: RepositoryKind::Local,
                identifier: "local-backup".to_string(),
                path: "/local".to_string(),
            },
            password: "pass".to_string(),
            include: vec![],
            exclude: vec![],
        };

        let client = reqwest::blocking::Client::new();
        let res = client.post("http://localhost:7667/backups")
            .body(serde_json::to_string(&backup).unwrap())
            .send().unwrap();
        println!("{:#?}", res);
        println!("{}", res.text().unwrap());
    });

    listbox.append(&add_backup);

    window
}
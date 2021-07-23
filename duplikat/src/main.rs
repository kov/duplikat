use std::str::FromStr;
use std::path::PathBuf;
use glib::clone;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use duplikat_types::*;
use strum::IntoEnumIterator;

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

    // Headerbar
    let headerbar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&headerbar));

    let run_button = gtk::Button::with_label("Run");
    run_button.connect_clicked(move |_| {
        let client = reqwest::blocking::Client::new();
        let res = client.post("http://localhost:7667/run")
            .send().unwrap();
        println!("{:#?}", res);
        println!("{}", res.text().unwrap());
    });

    // Create/edit backup
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);

    window.set_child(Some(&hbox));

    let grid = gtk::Grid::new();

    hbox.append(&grid);

    // Name
    let label = gtk::Label::new(Some("Name"));
    grid.attach(&label, 0, 0, 1, 1);

    let name_entry = gtk::Entry::new();
    name_entry.set_placeholder_text(Some("System backup"));

    grid.attach_next_to(&name_entry, Some(&label), gtk::PositionType::Right, 1, 1);

    // Type
    let label = gtk::Label::new(Some("Type"));
    grid.attach(&label, 0, 1, 1, 1);

    let type_combo = gtk::ComboBoxText::new();
    for kind in RepositoryKind::iter() {
        type_combo.append(Some(&kind.to_string()), kind.to_human_readable());
    }
    type_combo.set_active_id(Some("local"));

    grid.attach_next_to(&type_combo, Some(&label), gtk::PositionType::Right, 1, 1);

    // Identifier (bucket name, host, etc)
    let label = gtk::Label::new(None);
    label.set_visible(false);
    grid.attach(&label, 0, 2, 1, 1);

    let identifier = gtk::Entry::new();
    identifier.set_visible(false);

    grid.attach_next_to(&identifier, Some(&label), gtk::PositionType::Right, 1, 1);

    // Adjust entries based on type
    type_combo.connect_changed(
        clone!(@weak type_combo, @weak label, @weak identifier => move |_| {
            let repo_type = type_combo.active_id()
                .expect("Combo box should never be empty")
                .to_string();

            // Most types will need an identifier
            label.set_visible(true);
            identifier.set_visible(true);
            match RepositoryKind::from_str(repo_type.as_str()).unwrap() {
                RepositoryKind::Local => {
                    label.set_visible(false);
                    identifier.set_visible(false);
                },
                RepositoryKind::B2 => {
                    label.set_label("Bucket");
                    identifier.set_placeholder_text(Some("bucket-name"));
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
    grid.attach(&label, 0, 2, 1, 1);

    let path = gtk::Entry::new();
    path.set_text("/");

    grid.attach_next_to(&path, Some(&label), gtk::PositionType::Right, 1, 1);

    // Password
    let label = gtk::Label::new(Some("Password"));
    grid.attach(&label, 0, 3, 1, 1);

    let password = gtk::PasswordEntry::new();
    password.set_show_peek_icon(true);

    grid.attach_next_to(&password, Some(&label), gtk::PositionType::Right, 1, 1);

    // Password
    let label = gtk::Label::new(Some("Confirm Password"));
    grid.attach(&label, 0, 4, 1, 1);

    let confirm = gtk::PasswordEntry::new();
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

    add_backup.connect_clicked(move |_| {
        let repo_type = type_combo.active_id()
            .expect("Combo box should never be empty")
            .to_string();
        let identifier = identifier.text().to_string();
        let path = path.text().to_string();
        let password = password.text().to_string();

        let repository_str = format!("{}:{}:{}", repo_type, identifier, path);

        let backup = Backup {
            name: name_entry.text().to_string(),
            repository: Repository::from(repository_str.as_str()),
            password,
            include: vec![PathBuf::from("/home/kov/Downloads")],
            exclude: vec![".cache".to_string()],
        };

        let client = reqwest::blocking::Client::new();
        let res = client.post("http://localhost:7667/backups")
            .body(serde_json::to_string(&backup).unwrap())
            .send().unwrap();
        println!("{:#?}", res);
        println!("{}", res.text().unwrap());
    });

    grid.attach(&add_backup, 1, 5, 1, 1);

    window
}
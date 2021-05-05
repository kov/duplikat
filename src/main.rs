use gettextrs::*;
use gtk::prelude::*;
use duplikat_types::*;
use isahc::prelude::*;

mod config;
mod window;
use crate::window::Window;

async fn get_backups_list() {
    let mut res = Request::get("http://localhost:7667/backups")
        .body(())
        .unwrap()
        .send_async()
        .await;
    match &mut res {
        Ok(res) => {
            println!("{:#?}", res);
            println!("{:#?}", res.json::<Vec<Backup>>());
        },
        Err(e) => println!("Error: {:#?}", e),
    }
}

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("duplikat", config::LOCALEDIR);
    textdomain("duplikat");

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/duplikat.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    libadwaita::init();

    let app = gtk::Application::new(Some("br.dev.kov.Duplikat"), Default::default());

    app.connect_activate(move |app| {
        let window = Window::new();

        window.widget.set_application(Some(app));
        app.add_window(&window.widget);
        window.widget.present();

        glib::MainContext::default().spawn_local(
            get_backups_list()
        );
    });

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

    let ret = app.run();
    std::process::exit(ret);
}

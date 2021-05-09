use vial::prelude::*;
use duplikat_types::*;

routes! {
    GET "/backups" => list_backups;
    POST "/backups" => create_backup;
}

fn list_backups(_: Request) -> impl Responder {
    let backups = vec![
        Backup {
            name: "uva".to_string(),
            repository: Repository {
                kind: RepositoryKind::B2,
                identifier: "fedora-vm-uva".to_string(),
                path: "/system".to_string(),
            },
            password: "pass".to_string()
        },
        Backup {
            name: "pera".to_string(),
            repository: Repository {
                kind: RepositoryKind::B2,
                identifier: "mini-m1-pera".to_string(),
                path: "/system".to_string(),
            },
            password: "pass".to_string()
        }
    ];
    Response::from(200)
        .with_json(backups)
}

fn create_backup(req: Request) -> impl Responder {
    let backup = req.json::<Backup>().unwrap();
    println!("{:#?}", backup);
    Response::from(200)
        .with_json(backup)
}

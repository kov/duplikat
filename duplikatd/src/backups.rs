use duplikat_types::*;
use std::fs::File;
use std::io::{Result, prelude::*};
use std::path::Path;
use vial::prelude::*;
struct Configuration {}

impl Configuration {
    pub fn create(backup: &Backup) -> Result<()> {
        let mut base_path = Self::base_config_path();
        base_path.push("duplikatd");
        base_path.push("backups");
        base_path.push(&backup.name);

        std::fs::create_dir_all(&*base_path.to_string_lossy())?;

        let repository = "lala";
        Self::write_str_to_file(&base_path, repository)?;

        Ok(())
    }

    fn write_str_to_file(base_path: &Path, data: &str) -> Result<()> {
        let mut file_path = base_path.to_path_buf();
        file_path.push("repo");

        let mut file = File::create(&*file_path.to_string_lossy())?;
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    fn base_config_path() -> std::path::PathBuf {
        match users::get_effective_uid() {
            0 => {
                let mut base_path = std::path::PathBuf::from("/");
                base_path.push("etc");
                base_path
            },
            _ => {
                dirs::config_dir().unwrap()
            }
        }
    }
}

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
            password: "pass".to_string(),
            include: vec![],
            exclude: vec![],
        },
        Backup {
            name: "pera".to_string(),
            repository: Repository {
                kind: RepositoryKind::B2,
                identifier: "mini-m1-pera".to_string(),
                path: "/system".to_string(),
            },
            password: "pass".to_string(),
            include: vec![],
            exclude: vec![],
        }
    ];
    Response::from(200)
        .with_json(backups)
}

fn create_backup(req: Request) -> impl Responder {
    let backup = req.json::<Backup>().unwrap();
    println!("{:#?}", backup);

    let configuration = Configuration::create(&backup);
    println!("{:#?}", configuration);

    Response::from(200)
        .with_json(backup)
}

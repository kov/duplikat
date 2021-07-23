use duplikat_types::*;
use std::fs::File;
use std::io::{Result, prelude::*};
use std::path::{Path, PathBuf};
use vial::prelude::*;
use crate::restic::Restic;

pub struct Configuration {}

impl Configuration {
    pub fn create(backup: &Backup) -> Result<()> {
        let mut base_path = Self::base_config_path();
        base_path.push("duplikatd");
        base_path.push("backups");
        base_path.push(&backup.name);

        std::fs::create_dir_all(&*base_path.to_string_lossy())?;

        let repository_string = backup.repository.to_string();
        Self::write_str_to_file(&base_path, "repo", &repository_string)?;
        Self::write_str_to_file(&base_path, "password", &backup.password)?;
        Self::write_include_file(&base_path, &backup.include)?;
        Self::write_exclude_file(&base_path, &backup.exclude)?;

        Ok(())
    }

    fn write_str_to_file(base_path: &Path, filename: &str, data: &str) -> Result<()> {
        let mut file_path = base_path.to_path_buf();
        file_path.push(filename);

        let mut file = File::create(&*file_path.to_string_lossy())?;
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    fn write_include_file(base_path: &Path, include: &[PathBuf]) -> Result<()> {
        let mut file_path = base_path.to_path_buf();
        file_path.push("include");

        let mut file = File::create(&*file_path.to_string_lossy())?;
        for path in include {
            file.write_all(path.to_string_lossy().as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }

        Ok(())
    }

    fn write_exclude_file(base_path: &Path, exclude: &[String]) -> Result<()> {
        let mut file_path = base_path.to_path_buf();
        file_path.push("exclude");

        let mut file = File::create(&*file_path.to_string_lossy())?;
        for pattern in exclude {
            file.write_all(pattern.as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }

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

    fn config_file(name: &str, filename: &str) -> std::path::PathBuf {
        let mut path = Self::base_config_path();
        path.push("duplikatd");
        path.push("backups");
        path.push(name);
        path.push(filename);
        path
    }

    pub fn repo_file(name: &str) -> std::path::PathBuf {
        Self::config_file(name, "repo")
    }

    pub fn password_file(name: &str) -> std::path::PathBuf {
        Self::config_file(name, "password")
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

    if let Ok(()) = Restic::create_repo(&backup.name) {
        Response::from(200)
            .with_json(backup)
    } else {
        Response::from(500)
    }

}

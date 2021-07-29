use std::fs::File;
use std::io::{prelude::*, BufRead, BufReader, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use duplikat_types::*;
use tokio::io::{AsyncWriteExt};
use tokio::net::tcp::WriteHalf;

pub(crate) struct Restic {}

impl Restic {
    // FIXME: need to propagate error here so that we can tell the client.
    pub(crate) async fn create_backup(backup: &Backup) {
        if let Err(error) = Configuration::create(&backup) {
            println!("Failed to create backup! {:#?}", error);
        };

        if let Err(error) = Restic::create_repo(&backup.name).await {
            println!("Failed to create backup! {:#?}", error);
        }
    }

    pub(crate) async fn create_repo(name: &str) -> Result<()> {
        Command::new("restic")
            .args(&[
                "--json",
                "init",
                "--repository-file", &Configuration::repo_file(name).to_string_lossy(),
                "--password-file", &Configuration::password_file(name).to_string_lossy(),
            ])
            .output()
            .map(|_| ()) // FIXME: proper error handling will require looking into Output
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn run_backup<'a>(name: &str, writer: &mut WriteHalf<'a>) {
        let child = Command::new("restic")
            .args(&[
                "--json",
                "backup",
                "--files-from", &Configuration::include_file(name).to_string_lossy(),
                "--exclude-file", &Configuration::exclude_file(name).to_string_lossy(),
                "--repository-file", &Configuration::repo_file(name).to_string_lossy(),
                "--password-file", &Configuration::password_file(name).to_string_lossy(),
            ])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to run restic");

        let output = child.stdout.expect("Failed to open stdout");
        let output = BufReader::new(output);
        for line in output.lines() {
            let mut line = line.unwrap();
            line.push('\n');
            writer.write_all(line.as_bytes()).await.unwrap();
        }
    }
}

pub struct Configuration {}

impl Configuration {
    pub fn create(backup: &Backup) -> Result<()> {
        let mut base_path = Self::base_config_path();
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
        let mut base_path = match users::get_effective_uid() {
            0 => {
                let mut base_path = std::path::PathBuf::from("/");
                base_path.push("etc");
                base_path
            },
            _ => {
                dirs::config_dir().unwrap()
            }
        };
        base_path.push("duplikatd");
        base_path.push("backups");
        base_path
    }

    fn config_file(name: &str, filename: &str) -> std::path::PathBuf {
        let mut path = Self::base_config_path();
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

    pub fn include_file(name: &str) -> std::path::PathBuf {
        Self::config_file(name, "include")
    }

    pub fn exclude_file(name: &str) -> std::path::PathBuf {
        Self::config_file(name, "exclude")
    }
}
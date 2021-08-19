use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Error, Result, bail};
use duplikat_types::*;
use futures::future::join_all;
use log::{error,warn};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::WriteHalf;

pub(crate) struct Restic {}

impl Restic {
    #[allow(clippy::needless_lifetimes)]
    pub(crate) async fn create_backup<'a>(backup: &Backup, writer: &mut WriteHalf<'a>) {
        if let Err(error) = Configuration::create(&backup) {
            send_json(
                &json!({
                    "error": Some(ServerError::Configuration(
                        error.to_string())
                    )
                }),
                writer
            ).await;
            return;
        }

        if let Err(error) = Restic::create_repo(&backup.name).await {
            dbg!(&error);
            send_json(
                &json!({
                    "error": Some(ServerError::RepoInit(
                        error.to_string().trim().to_string())
                    )
                }),
                writer
            ).await;
            Configuration::remove(&backup.name).await;
            return;
        }

        send_json(
            &json!({
                "message": "OK"
            }),
            writer
        ).await;
    }

    pub(crate) async fn create_repo(name: &str) -> Result<()> {
        let environment = Configuration::environment_for_name(name).await;
        dbg!(&environment);
        let child = Command::new("restic")
            .args(&[
                "--json",
                "init",
                "--repository-file", &Configuration::repo_file(name).to_string_lossy(),
                "--password-file", &Configuration::password_file(name).to_string_lossy(),
            ])
            .envs(environment)
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to run restic");

        let mut stderr = child.stderr.expect("Failed to open stderr");
        let mut output = String::new();
        stderr.read_to_string(&mut output).unwrap();
        if !output.is_empty() {
            dbg!(&output);
            bail!(output)
        }
        dbg!("OK?");
        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn run_backup<'a>(name: &str, writer: &mut WriteHalf<'a>) {
        let environment = Configuration::environment_for_name(name).await;
        let child = Command::new("restic")
            .args(&[
                "--json",
                "backup",
                "--files-from", &Configuration::include_file(name).to_string_lossy(),
                "--exclude-file", &Configuration::exclude_file(name).to_string_lossy(),
                "--repository-file", &Configuration::repo_file(name).to_string_lossy(),
                "--password-file", &Configuration::password_file(name).to_string_lossy(),
            ])
            .envs(environment)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to run restic");

        let output = child.stdout.expect("Failed to open stdout");
        let output = BufReader::new(output);
        for line in output.lines() {
            let mut line = line.unwrap();
            line.push('\n');
            if let Err(error) = writer.write_all(line.as_bytes()).await {
                error!("{:#?}", error);
            }
        }
    }

    pub(crate) async fn stats_for(name: String) -> Result<(String, String)> {
        let environment = Configuration::environment_for_name(&name).await;
        let child = Command::new("restic")
            .args(&[
                "--json",
                "stats",
                "--repository-file", &Configuration::repo_file(&name).to_string_lossy(),
                "--password-file", &Configuration::password_file(&name).to_string_lossy(),
            ])
            .envs(environment)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to run restic");

        let output = child.stdout.expect("Failed to open stdout");
        let output = BufReader::new(output);

        // Stats only return a single line that looks like this:
        // {"total_size":2349097,"total_file_count":8}
        output.lines()
            .collect::<std::io::Result<Vec<String>>>()
            .map_err(Error::new)
            .map(|mut lines| {
                assert_eq!(lines.len(), 1);
                (name, lines.pop().unwrap())
            })
    }
}

pub struct Configuration {}

impl Configuration {
    pub async fn backup_with_name(name: &str) -> Result<Backup> {
        let mut base_path = Self::base_config_path();
        base_path.push(name);

        // Ensure it's a directory?
        let _ = tokio::fs::metadata(base_path.as_path()).await?;

        let repository = Repository::from(
            Self::read_file(
                Self::repo_file(name).as_path()
            ).unwrap().as_str()
        );

        let password = Self::read_file(
            Self::password_file(name).as_path()
        ).unwrap();


        let include = Self::read_file_to_vec::<PathBuf>(
            Self::include_file(name).as_path()
        ).unwrap();

        let exclude = Self::read_file_to_vec::<String>(
            Self::exclude_file(name).as_path()
        ).unwrap();

        Ok(Backup {
            name: name.to_string(),
            repository,
            password,
            key_id: None,
            key_secret: None,
            include,
            exclude,
        })
    }

    fn read_file(path: &Path) -> Result<String> {
        let mut repo_file = File::open(path)?;
        let mut contents = String::new();
        repo_file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn read_file_to_vec<T>(path: &Path) -> Result<Vec<T>>
    where T: From<String>
    {
        let mut repo_file = BufReader::new(
            File::open(path)?
        );
        let mut contents = String::new();
        let mut vector = vec![];
        while repo_file.read_line(&mut contents)? != 0 {
            vector.push(T::from(contents.trim().to_string()));
            contents.clear();
        }
        Ok(vector)
    }

    pub fn create(backup: &Backup) -> Result<()> {
        let mut base_path = Self::base_config_path();
        base_path.push(&backup.name);

        std::fs::create_dir_all(&*base_path.to_string_lossy())?;

        let repository_string = backup.repository.to_string();
        Self::write_str_to_file(&base_path, "repo", &repository_string)?;
        Self::write_str_to_file(&base_path, "password", &backup.password)?;
        Self::write_include_file(&base_path, &backup.include)?;
        Self::write_exclude_file(&base_path, &backup.exclude)?;

        match backup.repository.kind {
            RepositoryKind::B2 => {
                Self::write_str_to_file(
                    &base_path, "environment",
                    &format!(
                        "B2_ACCOUNT_ID={}\nB2_ACCOUNT_KEY={}\n",
                        backup.key_id.as_ref().unwrap(),
                        backup.key_secret.as_ref().unwrap(),
                    )
                )?;
            },
            _ => ()
        }

        Ok(())
    }

    pub(crate) async fn remove(name: &str) {
        let mut path = Self::base_config_path();
        path.push(name);

        tokio::fs::remove_dir_all(path.as_path()).await.unwrap();
    }

    pub(crate) async fn environment_for_name(name: &str) -> HashMap<String, String> {
        let mut environment = HashMap::<String, String>::new();

        let variables = match Self::read_file_to_vec::<String>(
            Self::environment_file(name).as_path()
        ) {
            Ok(v) => v,
            Err(_) => return environment, // FIXME: check if error is expected (not found) or not
        };

        for variable in variables {
            let (key, value) = variable.split_once("=").unwrap();
            environment.insert(key.to_string(), value.to_string());
        }

        environment
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn list<'a>(writer: &mut WriteHalf<'a>) {
        let base_path = Self::base_config_path();
        let mut entries = match tokio::fs::read_dir(base_path).await {
            Ok(entries) => entries,
            Err(error) => { warn!("{:#?}", error); return },
        };

        let mut backups = vec![];
        while let Some(entry) = entries.next_entry().await.unwrap() {
            backups.push(
                Self::backup_with_name(
                    entry.file_name()
                        .to_string_lossy()
                        .to_string()
                        .as_str()
                ).await.unwrap()
            );
        }

        let stats_futures: Vec<_> = backups.iter()
            .map(|backup| {
                Restic::stats_for(backup.name.clone())
            })
            .collect();

        let message = ResticMessage::BackupsList(
            ResticMessageBackupsList {
                list: backups,
            }
        );

        send_message(&message, writer).await;

        let lines = join_all(stats_futures).await;

        for line in lines {
            let (name, stats) = line.unwrap();
            dbg!(&Configuration::backup_with_name(&name).await);
            let line = add_message_type(
                &stats,
                "backupstats"
            );
            let mut line = add_key(
                &line,
                "name",
                name,
            );
            line.push('\n');
            writer.write_all(line.as_bytes()).await.unwrap();
        }
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

    pub fn environment_file(name: &str) -> std::path::PathBuf {
        Self::config_file(name, "environment")
    }
}

#[allow(clippy::needless_lifetimes)]
async fn send_message<'a>(message: &ResticMessage, writer: &mut WriteHalf<'a>) {
    writer.write_all(
        serde_json::to_string(&message)
            .unwrap()
            .as_bytes()
    ).await.unwrap();

    writer.write_all("\n".as_bytes()).await.unwrap();
}

#[allow(clippy::needless_lifetimes)]
async fn send_json<'a>(json: &serde_json::Value, writer: &mut WriteHalf<'a>) {
    writer.write_all(json.to_string().as_bytes()
    ).await.unwrap();

    writer.write_all("\n".as_bytes()).await.unwrap();
}

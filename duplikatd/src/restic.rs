use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use vial::prelude::*;
use crate::backups::Configuration;

pub(crate) struct Restic {}

impl Restic {
    pub(crate) fn create_repo(name: &str) -> Result<(), std::io::Error> {
        let output = Command::new("restic")
            .args(&[
                "--json",
                "init",
                "--repository-file", &Configuration::repo_file(name).to_string_lossy(),
                "--password-file", &Configuration::password_file(name).to_string_lossy(),
            ])
            .output()?;
        println!("{:#?}", output);
        Ok(())
    }
}

routes! {
    POST "/run" => run_backup;
}

fn run_backup(_req: Request) -> impl Responder {
    let name = "kov";
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
        println!("{}", line.unwrap());
    }
    Response::from(200)
}

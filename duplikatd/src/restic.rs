use std::process::Command;
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
    Response::from(200)
}

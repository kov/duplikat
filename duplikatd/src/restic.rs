use duplikat_types::Backup;
use std::fs::File;
use std::io::prelude::*;

pub struct Configuration {}

impl Configuration {
    pub fn create(backup: &Backup) -> std::io::Result<()> {
        let mut base_path = Self::base_config_path();
        base_path.push("duplikatd");
        base_path.push("backups");
        base_path.push(&backup.name);

        std::fs::create_dir_all(&*base_path.to_string_lossy())?;

        let mut config_path = base_path;
        config_path.push("config.json");

        let mut config = File::create(&*config_path.to_string_lossy())?;
        config.write_all(serde_json::to_string_pretty(&backup).unwrap().as_bytes())?;

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
use std::{path::PathBuf, str::FromStr};
use serde::{Serialize, Deserialize};
use strum_macros::{EnumIter, EnumString, ToString};

mod client;
mod server;
pub use crate::client::*;
pub use crate::server::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, EnumIter, EnumString, ToString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum RepositoryKind {
    Local,
    SFTP,
    B2,
}

impl RepositoryKind {
    #[allow(dead_code)]
    pub fn to_human_readable(&self) -> &str {
        match self {
            RepositoryKind::Local => "Local Directory",
            RepositoryKind::SFTP => "SFTP",
            RepositoryKind::B2 => "Backblaze B2",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Repository {
    pub kind: RepositoryKind,
    pub identifier: String,
    pub path: String,
}

impl ToString for Repository {
    fn to_string(&self) -> String {
        match self.kind {
            RepositoryKind::Local => self.path.clone(),
            _ => format!("{}:{}:{}",
                self.kind.to_string(), self.identifier, self.path
            ),
        }
    }
}

impl From<&str> for Repository {
    fn from(string: &str) -> Self {
        if string.starts_with('/') {
            Repository {
                kind: RepositoryKind::Local,
                identifier: "".to_string(),
                path: string.to_string(),
            }
        } else {
            let mut parts = string.split(':');

            let kind = parts.next()
                .expect("Malformed repository string: expected ':', not found");
            let identifier = parts.next()
                .expect("Malformed repository string: no identifier found")
                .to_string();
            let path = parts.next()
                .expect("Malformed repository string: no path")
                .to_string();

            let kind = RepositoryKind::from_str(kind)
                .unwrap_or_else(|_| panic!("Bad kind name: {}", kind));

            Repository {
                kind,
                identifier,
                path,
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Backup {
    pub name: String,
    pub repository: Repository,
    pub key_id: Option<String>,
    pub key_secret: Option<String>,
    pub password: String,
    pub include: Vec<PathBuf>,
    pub exclude: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let backup = Backup {
            name: "test".to_string(),
            repository: Repository {
                kind: RepositoryKind::B2,
                identifier: "server-test".to_string(),
                path: "/system".to_string(),
            },
            key_id: Some("key-id".to_string()),
            key_secret: Some("very-secret".to_string()),
            password: "pass".to_string(),
            include: vec![],
            exclude: vec![],
        };

        assert_eq!(
            serde_json::to_string(&backup).unwrap(),
            "{\"name\":\"test\",\"repository\":{\"kind\":\"b2\",\"identifier\":\"server-test\",\"path\":\"/system\"},\"key_id\":\"key-id\",\"key_secret\":\"very-secret\",\"password\":\"pass\",\"include\":[],\"exclude\":[]}".to_string()
        )
    }

    #[test]
    fn b2_repository() {
        let repo_str_reference = "b2:back:/blaze";
        let repo_reference = Repository {
                kind: RepositoryKind::B2,
                identifier: "back".to_string(),
                path: "/blaze".to_string()
        };

        assert_eq!(
            repo_reference,
            Repository::from("b2:back:/blaze")
        );

        assert_eq!(
            repo_str_reference,
            repo_reference.to_string()
        );
     }

    #[test]
    fn local_repository() {
        let repo_str_reference = "/mnt/backup";
        let repo_reference = Repository {
                kind: RepositoryKind::Local,
                identifier: "".to_string(),
                path: "/mnt/backup".to_string()
        };

        assert_eq!(
            repo_reference,
            Repository::from("/mnt/backup")
        );

        assert_eq!(
            repo_str_reference,
            repo_reference.to_string()
        );
     }
}

pub fn add_message_type(json_string: &str, type_string: &str) -> String {
    add_key(json_string, "message_type", type_string.to_string())
}

pub fn add_key(json_string: &str, key: &str, value: String) -> String {
    let mut object_value: serde_json::Value = serde_json::from_str(json_string).unwrap();
    let object = object_value.as_object_mut().unwrap();
    object.insert(
        key.to_string(),
        serde_json::Value::String(value)
    );
    object_value.to_string()
}

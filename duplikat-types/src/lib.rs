use std::{path::PathBuf, str::FromStr};
use serde::{Serialize, Deserialize};
use strum_macros::{EnumIter, EnumString, ToString};

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
    pub password: String,
    pub include: Vec<PathBuf>,
    pub exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResticMessageStatus {
    pub percent_done: f64,
    pub total_files: u64,
    pub files_done: u64,
    pub total_bytes: u64,
    pub bytes_done: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResticMessageSummary{
    pub files_new: u64,
    pub files_changed: u64,
    pub files_unmodified: u64,
    pub dirs_new: u64,
    pub dirs_changed: u64,
    pub dirs_unmodified: u64,
    pub data_blobs: u64,
    pub tree_blobs: u64,
    pub data_added: u64,
    pub total_files_processed: u64,
    pub total_bytes_processed: u64,
    pub total_duration: f64,
    pub snapshot_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "message_type")]
pub enum ResticMessage {
    Status(ResticMessageStatus),
    Summary(ResticMessageSummary),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_works() {
        let backup = Backup {
            name: "test".to_string(),
            repository: Repository {
                kind: RepositoryKind::B2,
                identifier: "server-test".to_string(),
                path: "/system".to_string(),
            },
            password: "pass".to_string(),
            include: vec![],
            exclude: vec![],
        };

        assert_eq!(
            serde_json::to_string(&backup).unwrap(),
            "{\"name\":\"test\",\"repository\":{\"kind\":\"b2\",\"identifier\":\"server-test\",\"path\":\"/system\"},\"password\":\"pass\",\"include\":[],\"exclude\":[]}"
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

     #[test]
     fn restic_message() {
         let status_message_value = json!({
            "message_type": "status",
            "percent_done": 0.07672352397538289,
            "total_files": 5,
            "files_done": 2,
            "total_bytes": 2349097,
            "bytes_done": 180231
        });
        let status_message = ResticMessage::Status(ResticMessageStatus {
            percent_done: 0.07672352397538289,
            total_files: 5,
            files_done: 2,
            total_bytes: 2349097,
            bytes_done: 180231
        });
        let deserialized_message: ResticMessageStatus = serde_json::from_value(status_message_value).unwrap();
        if let ResticMessage::Status(status_message) = status_message {
            assert_eq!(status_message.total_files, deserialized_message.total_files);
            assert_eq!(status_message.total_bytes, deserialized_message.total_bytes);
        }
     }
}

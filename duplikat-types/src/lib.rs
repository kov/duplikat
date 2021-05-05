use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryKind {
    Local,
    SFTP,
    B2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub kind: RepositoryKind,
    pub identifier: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Backup {
    pub name: String,
    pub repository: Repository,
    pub password: String,
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
            password: "pass".to_string()
        };

        assert_eq!(
            serde_json::to_string(&backup).unwrap(),
            "{\"name\":\"test\",\"repository\":{\"kind\":\"b2\",\"identifier\":\"server-test\",\"path\":\"/system\"},\"password\":\"pass\"}"
        )
    }
}

use serde::{Serialize, Deserialize};
use crate::Backup;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessageCreateBackup {
    pub backup: Backup,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessageRunBackup {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "message_type")]
pub enum ClientMessage {
    CreateBackup(ClientMessageCreateBackup),
    ListBackups,
    RunBackup(ClientMessageRunBackup),
}

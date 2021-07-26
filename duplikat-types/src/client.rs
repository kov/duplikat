use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessageRunBackup {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "message_type")]
pub enum ClientMessage {
    RunBackup(ClientMessageRunBackup),
}

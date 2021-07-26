use serde::{Serialize, Deserialize};

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

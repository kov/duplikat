use duplikat_types::*;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, tcp::WriteHalf};
use restic::{Configuration, Restic};

mod restic;

#[allow(clippy::needless_lifetimes)]
async fn process_request<'a>(buffer: &str, writer: &mut WriteHalf<'a>) {
    if let Ok(result) = serde_json::from_str(buffer) {
        match result {
            ClientMessage::CreateBackup(create) => Restic::create_backup(&create.backup).await,
            ClientMessage::RunBackup(backup) => Restic::run_backup(&backup.name, writer).await,
            ClientMessage::ListBackups => Configuration::list(writer).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7667").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);

            let mut buffer = String::new();
            while let Ok(count) = reader.read_line(&mut buffer).await {
                if count == 0 {
                    break;
                }
                process_request(buffer.trim_end(), &mut writer).await;
                buffer.clear();
            };
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}

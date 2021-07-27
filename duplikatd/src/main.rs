mod backups;
mod restic;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, tcp::WriteHalf};
use backups as backups_routes;
use restic::run_backup;

mod index {
    vial::routes! {
        GET "/" => |_| "<h1>This is the index.</h1>";
    }
}

async fn process_request<'a>(buffer: &str, writer: &mut WriteHalf<'a>) {
    println!("{:#?}", buffer);
    run_backup("kov", writer).await;
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
                println!("{}: {:#?}", count, buffer);
                process_request(&buffer, &mut writer).await;
                buffer.clear();
            };
        });
    }

    vial::run!(
        index,
        backups_routes
    ).unwrap();

    Ok(())
}

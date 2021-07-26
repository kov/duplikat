mod backups;
mod restic;

use tokio::io::{AsyncBufReadExt, BufReader, BufWriter};
use tokio::net::TcpListener;
use backups as backups_routes;
use restic as restic_routes;

mod index {
    vial::routes! {
        GET "/" => |_| "<h1>This is the index.</h1>";
    }
}

async fn process_request<W>(buffer: &str, writer: &mut BufWriter<W>) {
    println!("{:#?}", buffer);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7667").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let (reader, writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            let mut buffer = String::new();
            while let Ok(count) = reader.read_line(&mut buffer).await {
                println!("{}: {:#?}", count, buffer);
                process_request(&buffer, &mut writer).await;
                buffer.clear();
            };
        });
    }

    vial::run!(
        index,
        backups_routes,
        restic_routes
    ).unwrap();

    Ok(())
}

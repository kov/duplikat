mod backups;
mod restic;

use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use backups as backups_routes;
use restic as restic_routes;

mod index {
    vial::routes! {
        GET "/" => |_| "<h1>This is the index.</h1>";
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7667").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }

    vial::run!(
        index,
        backups_routes,
        restic_routes
    ).unwrap();

    Ok(())
}

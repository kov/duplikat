use duplikat_types::*;
use glib::{prelude::*, error::Error as GError, source::Priority};
use gio::{prelude::*, SocketClient, DataInputStream, IOStream, OutputStream};
use gtk::prelude::*;

pub(crate) struct Server {}

impl Server {
    pub(crate) async fn connect() -> Result<(IOStream, OutputStream, DataInputStream), GError> {
        let socket = SocketClient::new();
        let result = socket.connect_to_host_async_future("127.0.0.1:7667", 7667).await;

        let socket = match result {
            Ok(s) => s,
            Err(error) => {
                let dialog = gtk::MessageDialogBuilder::new()
                    .message_type(gtk::MessageType::Error)
                    .buttons(gtk::ButtonsType::Close)
                    .text("Failed to connect to server.")
                    .secondary_text(&error.to_string())
                    .build();
                dialog.run_future().await;
                dialog.close();
                return Err(error);
            }
        };

        let stream = socket.upcast::<IOStream>();

        let writer = stream.output_stream();
        let reader = DataInputStream::new(&stream.input_stream());

        // stream needs to be returned here or we'll panic as it is actually the one
        // that owns writer and reader, but that doesn't seem to be properly conveyed
        // to Rust by the bindings.
        Ok((stream, writer, reader))
    }

    pub(crate) async fn send_message(message: ClientMessage) -> Result<(IOStream, OutputStream, DataInputStream), GError> {
        let (stream, writer, reader) = Server::connect().await?;

        let mut json_message = serde_json::to_string(
            &message
        ).unwrap();

        // We need to make sure there is a new line otherwise the other side, which is a
        // buffered reader which reads line by line, will not process the message.
        json_message.push('\n');

        writer.write_all_async_future(json_message, Priority::default()).await
            .map_err(|err| err.1)?;

        Ok((stream, writer, reader))
    }
}
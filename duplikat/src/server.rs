use duplikat_types::*;
use glib::{prelude::*, error::Error as GError, source::Priority};
use gio::{prelude::*, SocketClient, DataInputStream, IOStream, OutputStream};
use gtk::prelude::*;

pub(crate) struct Connection {
    pub stream: IOStream,
    pub ostream: OutputStream,
    pub istream: DataInputStream,
}

impl Connection {
    pub(crate) async fn send_message(&self, message: ClientMessage) -> Result<(), GError> {
        let mut json_message = serde_json::to_string(
            &message
        ).unwrap();

        // We need to make sure there is a new line otherwise the other side, which is a
        // buffered reader which reads line by line, will not process the message.
        json_message.push('\n');

        self.ostream.write_all_async_future(json_message, Priority::default()).await
            .map_err(|err| err.1)?;

        Ok(())
    }

    pub(crate) async fn read_message(&self) -> Result<Option<ResticMessage>, GError> {
        self.istream.read_line_utf8_async_future(Priority::default()).await
            .map(|line| {
                if let Some(line) = line {
                    serde_json::from_str(&line.to_string())
                        .unwrap_or_else(|_| panic!("Received broken message: {:#?}", line))
                } else {
                    None
                }
            })
    }

    pub(crate) async fn read_response(&self) -> Result<ServerResponse, GError> {
        let line = self.istream.read_line_utf8_async_future(Priority::default()).await?;
        Ok(serde_json::from_str(line.unwrap().to_string().as_str()).unwrap())
    }
}

pub(crate) struct Server {}

impl Server {
    pub(crate) async fn connect() -> Result<Connection, GError> {
        let socket = SocketClient::new();
        let result = socket.connect_to_host_async_future("127.0.0.1:7667", 7667).await;

        let socket = match result {
            Ok(s) => s,
            Err(error) => {
                let main_window = crate::get_main_window();
                let dialog = gtk::MessageDialogBuilder::new()
                    .transient_for(&main_window)
                    .modal(true)
                    .message_type(gtk::MessageType::Error)
                    .buttons(gtk::ButtonsType::Close)
                    .text("Failed to connect to server.")
                    .secondary_text(&error.to_string())
                    .build();
                dialog.run_future().await;
                dialog.close();
                std::process::exit(1);
            }
        };

        let stream = socket.upcast::<IOStream>();

        let ostream = stream.output_stream();
        let istream = DataInputStream::new(&stream.input_stream());

        // stream needs to be returned here or we'll panic as it is actually the one
        // that owns writer and reader, but that doesn't seem to be properly conveyed
        // to Rust by the bindings.
        Ok(Connection {
            stream,
            ostream,
            istream,
        })
    }
}

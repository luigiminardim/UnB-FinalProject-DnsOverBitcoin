mod message_buffer;

use super::message::Message;
use crate::dns::{handler::Handler, net::Request};
use message_buffer::{DatagramReader, DatagramWriter};
use tokio::net::UdpSocket;

/// Messages carried by UDP are restricted to 512 bytes.
const UDP_LENGTH_LIMIT: usize = 512;

#[derive(Debug)]
pub enum UdpListenerError {
    IoError(std::io::Error),
}

pub struct UdpListener {
    handler: Box<dyn Handler>,
}

impl UdpListener {
    pub fn new(handler: impl Handler) -> Self {
        UdpListener {
            handler: Box::new(handler),
        }
    }

    pub async fn listen(self) -> Result<(), UdpListenerError> {
        let socket = UdpSocket::bind("127.0.0.1:1053")
            .await
            .map_err(UdpListenerError::IoError)?;
        loop {
            let mut buffer = [0; UDP_LENGTH_LIMIT];
            let (message_length, src_addresss) = socket
                .recv_from(&mut buffer)
                .await
                .map_err(UdpListenerError::IoError)?;
            let request_message = DatagramReader::new(&buffer[..message_length])
                .read_message()
                .unwrap();
            let id = request_message.id();
            let response_message = match request_message.into_request() {
                Ok(Request::Query(query_request)) => {
                    self.handler.handle_query(query_request).await.map_or_else(
                        |error| Message::from_handler_error(id, error),
                        |response| Message::from_query_reponse(&request_message, &response),
                    )
                }
                Err(response_message) => response_message,
            };
            let mut buffer = [0; UDP_LENGTH_LIMIT];
            let datagram_length = DatagramWriter::new(&mut buffer)
                .write_message(&response_message)
                .unwrap();
            socket
                .send_to(&buffer[..datagram_length], src_addresss)
                .await
                .map_err(UdpListenerError::IoError)?;
        }
    }
}

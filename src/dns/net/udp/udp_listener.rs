use super::{DatagramReader, DatagramWriter};
use crate::dns::resolver::Resolver;
use tokio::net::UdpSocket;

/// Messages carried by UDP are restricted to 512 bytes.
const UDP_LENGTH_LIMIT: usize = 512;

#[derive(Debug)]
pub enum UdpListenerError {
    IoError(std::io::Error),
}

pub struct UdpListener {
    handler: Box<dyn Resolver>,
}

impl UdpListener {
    pub fn new(handler: impl Resolver) -> Self {
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
            let request = DatagramReader::new(&buffer[..message_length])
                .read_message()
                .unwrap();
            let response = self.handler.resolve(&request).await;
            let mut buffer = [0; UDP_LENGTH_LIMIT];
            let datagram_length = DatagramWriter::new(&mut buffer)
                .write_message(&response)
                .unwrap();
            socket
                .send_to(&buffer[..datagram_length], src_addresss)
                .await
                .map_err(UdpListenerError::IoError)?;
        }
    }
}

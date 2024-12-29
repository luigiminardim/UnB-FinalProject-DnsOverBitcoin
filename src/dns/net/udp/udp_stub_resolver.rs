use crate::dns::{
    core::{Message, ResponseCode},
    resolver::Resolver,
};
use async_trait::async_trait;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

use super::{DatagramReader, DatagramWriter, UDP_LENGTH_LIMIT};

pub struct UdpStubResolver {
    socket_address: SocketAddr,
}

#[async_trait]
impl Resolver for UdpStubResolver {
    async fn resolve(&self, request: &Message) -> Message {
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0))
            .await
            .unwrap();
        let mut request_buffer = [0_u8; UDP_LENGTH_LIMIT];
        match DatagramWriter::new(&mut request_buffer).write_message(&request) {
            None => {
                eprintln!("Failed to write message");
                return request.into_response(ResponseCode::ServerFailure);
            }
            Some(request_datagram_length) => {
                if let Err(err) = socket
                    .send_to(
                        &request_buffer[..request_datagram_length],
                        self.socket_address,
                    )
                    .await
                {
                    eprintln!("Failed to send message: {:?}", err);
                    return request.into_response(ResponseCode::ServerFailure);
                }
            }
        }
        let mut response_buffer = [0_u8; UDP_LENGTH_LIMIT];
        match socket.recv_from(&mut response_buffer).await {
            Err(e) => {
                eprintln!("Failed to receive message: {:?}", e);
                return request.into_response(ResponseCode::ServerFailure);
            }
            Ok((response_datagram_length, _)) => {
                match DatagramReader::new(&response_buffer[..response_datagram_length])
                    .read_message()
                {
                    Some(response) => {
                        return response;
                    }
                    None => {
                        eprintln!("Failed to read message");
                        return request.into_response(ResponseCode::ServerFailure);
                    }
                }
            }
        }
    }
}

impl UdpStubResolver {
    pub fn new(socket_address: SocketAddr) -> Self {
        UdpStubResolver { socket_address }
    }
}

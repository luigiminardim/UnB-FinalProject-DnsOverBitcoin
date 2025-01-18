use std::{
    net::IpAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use hickory_server::{
    authority::Authority,
    authority::AuthorityObject,
    authority::Catalog,
    authority::MessageResponseBuilder,
    proto::{
        op::{Header, MessageType, OpCode, ResponseCode},
        rr::{
            rdata::{A, TXT},
            LowerName, Name, RData, Record,
        },
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
    ServerFuture,
};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let handler = Handler::new();
    let mut server = ServerFuture::new(handler);
    server.register_socket(UdpSocket::bind("0.0.0.0:1053").await.unwrap());
    server.block_until_done().await.unwrap();
}

#[derive(Clone, Debug)]
pub struct Handler {
    /// Request counter, incremented on every successful request.
    pub counter: Arc<AtomicU64>,
    /// Domain to serve DNS responses for (requests for other domains are silently ignored).
    pub root_zone: LowerName,
    /// Zone name for counter (counter.nostr-dns)
    pub counter_zone: LowerName,
    /// Zone name for myip (myip.nostr-dns)
    pub myip_zone: LowerName,
    /// Zone name for hello (hello.nostr-dns)
    pub hello_zone: LowerName,
}

impl Handler {
    /// Create new handler from command-line options.
    pub fn new() -> Self {
        Handler {
            counter: Arc::new(AtomicU64::new(0)),
            root_zone: "nostr-dns.".parse().unwrap(),
            counter_zone: "counter.nostr-dns.".parse().unwrap(),
            myip_zone: "myip.nostr-dns.".parse().unwrap(),
            hello_zone: "hello.nostr-dns.".parse().unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response: R,
    ) -> ResponseInfo {
        match self.do_handle_request(request, response.clone()).await {
            Ok(info) => {
                dbg!(info.op_code());
                info
            }
            Err(_) => {
                let builder = MessageResponseBuilder::from_message_request(request);
                let mut response_header = Header::response_from_request(request.header());
                response_header.set_response_code(ResponseCode::ServFail);
                let response_message = builder.build(response_header.clone(), &[], &[], &[], &[]);
                response.send_response(response_message).await.unwrap();
                ResponseInfo::from(response_header)
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidOpCode(OpCode),
    InvalidMessageType(MessageType),
    InvalidZone(LowerName),
    Io(std::io::Error),
}

impl Handler {
    /// Handle request, returning ResponseInfo if response was successfully sent, or an error.
    async fn do_handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> Result<ResponseInfo, Error> {
        // make sure the request is a query
        if request.op_code() != OpCode::Query {
            return Err(Error::InvalidOpCode(request.op_code()));
        }

        // make sure the message type is a query
        if request.message_type() != MessageType::Query {
            return Err(Error::InvalidMessageType(request.message_type()));
        }

        match request.query().name() {
            name if self.myip_zone.zone_of(name) => {
                dbg!("Aqui1");
                self.do_handle_request_myip(request, response).await
            }
            name if self.counter_zone.zone_of(name) => {
                dbg!("Aqui2");
                self.do_handle_request_counter(request, response).await
            }
            name if self.hello_zone.zone_of(name) => {
                dbg!("Aqui3");
                self.do_handle_request_hello(request, response).await
            }
            name => {
                dbg!("Aqui");
                Err(Error::InvalidZone(name.clone()))
            }
        }
    }

    async fn do_handle_request_myip<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Error> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);
        let rdata = match request.src().ip() {
            IpAddr::V4(ipv4) => RData::A(ipv4.into()),
            IpAddr::V6(ipv6) => RData::AAAA(ipv6.into()),
        };
        let records = vec![Record::from_rdata(request.query().name().into(), 60, rdata)];
        let response = builder.build(header, records.iter(), &[], &[], &[]);
        Ok(responder.send_response(response).await.map_err(Error::Io)?)
    }

    /// Handle requests for counter.{domain}.
    async fn do_handle_request_counter<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Error> {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);
        let rdata = RData::TXT(TXT::new(vec![counter.to_string()]));
        let records = vec![Record::from_rdata(request.query().name().into(), 60, rdata)];
        let response = builder.build(header, records.iter(), &[], &[], &[]);
        Ok(responder.send_response(response).await.map_err(Error::Io)?)
    }

    /// Handle requests for *.hello.{domain}.
    async fn do_handle_request_hello<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Error> {
        dbg!("Aqui Será?");
        self.counter.fetch_add(1, Ordering::SeqCst);
        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);
        let name: &Name = &request.query().name().into();
        let zone_parts = (name.num_labels() - self.hello_zone.num_labels() - 1) as usize;
        let name = name
            .iter()
            .enumerate()
            .filter(|(i, _)| i <= &zone_parts)
            .fold(String::from("hello,"), |a, (_, b)| {
                a + " " + &String::from_utf8_lossy(b)
            });
        let rdata = RData::TXT(TXT::new(vec![name]));
        let records = vec![Record::from_rdata(request.query().name().into(), 60, rdata)];
        let response = builder.build(header, records.iter(), &[], &[], &[]);
        Ok(responder.send_response(response).await.map_err(Error::Io)?)
    }
}

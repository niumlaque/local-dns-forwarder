use crate::dns;
use crate::resolve_event::{DefaultResolveEvent, ResolveEvent};
use crate::resolved_status::ResolvedStatus;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    address: String,
    port: u16,
    default_dns_server: Ipv4Addr,
}

impl Config {
    pub fn new(address: impl Into<String>, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
            default_dns_server: Ipv4Addr::new(8, 8, 8, 8),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".into(),
            port: 53,
            default_dns_server: Ipv4Addr::new(8, 8, 8, 8),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Address: {}, Port: {}, Default DNS Server: {}",
            self.address, self.port, self.default_dns_server
        )
    }
}

pub struct ServerBuilder<E: ResolveEvent> {
    config: Config,
    allowlist: HashMap<String, ()>,
    event: E,
}

impl<E: ResolveEvent> ServerBuilder<E> {
    pub fn build(self) -> Runner<E> {
        let default_dns_server = self.config.default_dns_server;
        Runner {
            config: self.config,
            default_dns_server: Arc::new(RwLock::new(default_dns_server)),
            event: self.event,
            allowlist: Arc::new(RwLock::new(self.allowlist)),
        }
    }

    pub fn allowlist(self, allowlist: HashMap<String, ()>) -> Self {
        Self {
            config: self.config,
            allowlist,
            event: self.event,
        }
    }
}

pub struct ServerConfigBuilder {
    config: Config,
    allowlist: HashMap<String, ()>,
}

impl ServerConfigBuilder {
    pub fn event<E: ResolveEvent>(self, event: E) -> ServerBuilder<E> {
        ServerBuilder {
            config: self.config,
            event,
            allowlist: self.allowlist,
        }
    }

    pub fn allowlist(self, allowlist: HashMap<String, ()>) -> Self {
        Self {
            config: self.config,
            allowlist,
        }
    }

    pub fn build(self) -> Runner<DefaultResolveEvent> {
        self.event(DefaultResolveEvent {}).build()
    }
}

pub struct Server;
impl Server {
    pub fn from_config(config: Config) -> ServerConfigBuilder {
        ServerConfigBuilder {
            config,
            allowlist: HashMap::new(),
        }
    }
}

pub struct Runner<E: ResolveEvent> {
    config: Config,
    default_dns_server: Arc<RwLock<Ipv4Addr>>,
    event: E,
    allowlist: Arc<RwLock<HashMap<String, ()>>>,
}

impl<E: ResolveEvent> Runner<E> {
    pub fn serve(&self) -> dns::Result<()> {
        let socket = UdpSocket::bind((&self.config.address as &str, self.config.port))?;
        loop {
            match self.on_recv(&socket) {
                Ok(_) => (),
                Err(e) => self.event.error(format!("{e}")),
            }
        }
    }

    fn on_recv(&self, socket: &UdpSocket) -> dns::Result<()> {
        let mut req_buffer = dns::BytePacketBuffer::new();
        let (_, src) = socket.recv_from(&mut req_buffer.buf)?;
        let mut req = dns::Message::read(&mut req_buffer)?;
        let mut resp = dns::Message::new();
        resp.header.id = req.header.id;
        resp.header.recursion_desired = true;
        resp.header.recursion_available = true;
        resp.header.response = true;

        if let Some(question) = req.questions.pop() {
            let qtype = question.qtype;
            let name = question.name.clone();
            let status = if self.check_allowlist(&question.name) {
                let dns_server = if let Ok(dds) = self.default_dns_server.read() {
                    *dds
                } else {
                    self.config.default_dns_server
                };

                if let Ok(result) = dns::lookup(dns_server, &question.name, question.qtype) {
                    resp.questions.push(question);
                    resp.header.rescode = result.header.rescode;
                    let mut addresses = Vec::with_capacity(result.answers.len());

                    for rec in result.answers {
                        match rec.rdata {
                            dns::RData::A(v) => addresses.push(IpAddr::V4(v)),
                            dns::RData::AAAA(v) => addresses.push(IpAddr::V6(v)),
                            _ => (),
                        }
                        resp.answers.push(rec);
                    }
                    for rec in result.authorities {
                        resp.authorities.push(rec);
                    }
                    for rec in result.resources {
                        resp.resources.push(rec);
                    }

                    if result.header.rescode == dns::ResultCode::NoError {
                        ResolvedStatus::Allow(qtype, name.clone(), addresses)
                    } else {
                        ResolvedStatus::AllowButError(qtype, name.clone(), result.header.rescode)
                    }
                } else {
                    resp.header.rescode = dns::ResultCode::ServFail;
                    ResolvedStatus::AllowButError(qtype, name.clone(), resp.header.rescode)
                }
            } else {
                resp.header.rescode = dns::ResultCode::Refused;
                ResolvedStatus::Deny(qtype, name.clone(), resp.header.rescode)
            };
            self.event.resolved(status);
        } else {
            resp.header.rescode = dns::ResultCode::FormErr;
            self.event
                .error(format!("{}: {}", resp.header.id, resp.header.rescode));
        }

        let mut resp_buffer = dns::BytePacketBuffer::new();
        resp.write(&mut resp_buffer)?;
        let len = resp_buffer.pos();
        let data = resp_buffer.get_range(0, len)?;
        socket.send_to(data, src)?;

        Ok(())
    }

    fn check_allowlist(&self, name: &str) -> bool {
        if let Ok(allowlist) = self.allowlist.read() {
            allowlist.contains_key(name)
        } else {
            self.event
                .error("Failed to get allow list(read lock error)");
            false
        }
    }
}

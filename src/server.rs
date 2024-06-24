use crate::allow_deny_list::{AllowDenyList, CheckStatus};
use crate::dns;
use crate::resolve_event::{DefaultResolveEvent, ResolveEvent};
use crate::resolved_status::ResolvedStatus;
use serde::Deserialize;
use std::fmt::Display;
use std::net::{Ipv4Addr, UdpSocket};
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
    checklist: AllowDenyList,
    event: E,
}

impl<E: ResolveEvent> ServerBuilder<E> {
    pub fn build(self) -> Runner<E> {
        let default_dns_server = self.config.default_dns_server;
        Runner {
            config: self.config,
            default_dns_server: Arc::new(RwLock::new(default_dns_server)),
            event: self.event,
            checklist: Arc::new(RwLock::new(self.checklist)),
        }
    }

    pub fn checklist(self, checklist: AllowDenyList) -> Self {
        Self {
            config: self.config,
            checklist,
            event: self.event,
        }
    }
}

pub struct ServerConfigBuilder {
    config: Config,
    checklist: AllowDenyList,
}

impl ServerConfigBuilder {
    pub fn event<E: ResolveEvent>(self, event: E) -> ServerBuilder<E> {
        ServerBuilder {
            config: self.config,
            event,
            checklist: self.checklist,
        }
    }

    pub fn checklist(self, checklist: AllowDenyList) -> Self {
        Self {
            config: self.config,
            checklist,
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
            checklist: Default::default(),
        }
    }
}

pub struct Runner<E: ResolveEvent> {
    config: Config,
    default_dns_server: Arc<RwLock<Ipv4Addr>>,
    event: E,
    pub checklist: Arc<RwLock<AllowDenyList>>,
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
        let mut raw_buf = Vec::new();

        if let Some(question) = req.questions.pop() {
            let qtype = question.qtype;
            let name = question.name.clone();
            if question.qtype == dns::QueryType::A || question.qtype == dns::QueryType::AAAA {
                match self.check(&question.name) {
                    CheckStatus::Deny => {
                        // Ignore FQDNs that are registered in the deny list
                        let (_, resp_buffer) =
                            Self::make_error_resp_msg(&req, dns::ResultCode::NXDomain)?;
                        raw_buf.extend(resp_buffer.get_all()?);
                    }
                    CheckStatus::Allow => {
                        let status = self.lookup(req.header.id, question, &mut raw_buf)?;
                        self.event.resolved(status);
                    }
                    CheckStatus::NotFound => {
                        let (resp, resp_buffer) =
                            Self::make_error_resp_msg(&req, dns::ResultCode::NXDomain)?;
                        raw_buf.extend(resp_buffer.get_all()?);
                        let res_data = crate::resolved_data::ResolvedData::new(qtype, name);
                        self.event
                            .resolved(ResolvedStatus::Deny(res_data, resp.header.rescode));
                    }
                }
            } else {
                let status = self.lookup(req.header.id, question, &mut raw_buf)?;
                self.event.resolved(status.into_nocheck());
            }
        } else {
            let (resp, resp_buffer) = Self::make_error_resp_msg(&req, dns::ResultCode::FormErr)?;
            raw_buf.extend(resp_buffer.get_all()?);
            self.event
                .error(format!("{}: {}", req.header.id, resp.header.rescode));
        }

        socket.send_to(&raw_buf, src)?;

        Ok(())
    }

    fn check(&self, name: &str) -> CheckStatus {
        if let Ok(checklist) = self.checklist.read() {
            checklist.check(name)
        } else {
            self.event
                .error("Failed to get allow list(read lock error)");
            CheckStatus::Deny
        }
    }

    fn lookup(
        &self,
        id: u16,
        question: dns::Question,
        raw: &mut Vec<u8>,
    ) -> dns::Result<ResolvedStatus> {
        let dns_server = if let Ok(dds) = self.default_dns_server.read() {
            *dds
        } else {
            self.config.default_dns_server
        };

        let mut res_data =
            crate::resolved_data::ResolvedData::new(question.qtype, question.name.clone());

        let ret = if let Ok((resp_buf, result)) = dns::lookup(
            dns_server,
            id,
            &question.name,
            question.qtype,
            question.class,
        ) {
            *raw = resp_buf;

            for rec in result.answers {
                match &rec.rdata {
                    dns::RData::A(v) => res_data.insert(dns::QueryType::A, v.to_string()),
                    dns::RData::AAAA(v) => res_data.insert(dns::QueryType::AAAA, v.to_string()),
                    dns::RData::CNAME(_, v, _) => res_data.insert(dns::QueryType::CNAME, v),
                    dns::RData::SRV(_, v, _) => res_data.insert(dns::QueryType::SRV, v.to_string()),
                    dns::RData::Unknown(qtype, _) => {
                        res_data.insert(dns::QueryType::UNKNOWN((*qtype).into()), "".to_string())
                    }
                }
            }

            if result.header.rescode == dns::ResultCode::NoError {
                ResolvedStatus::Allow(res_data)
            } else {
                ResolvedStatus::AllowButError(res_data, result.header.rescode)
            }
        } else {
            ResolvedStatus::AllowButError(res_data, dns::ResultCode::ServFail)
        };
        Ok(ret)
    }

    fn make_error_resp_msg(
        req: &dns::Message,
        result_code: dns::ResultCode,
    ) -> dns::Result<(dns::Message, dns::BytePacketBuffer)> {
        let mut resp = dns::Message::new();
        resp.header.id = req.header.id;
        resp.header.recursion_desired = req.header.recursion_desired;
        resp.header.recursion_available = req.header.recursion_available;
        resp.header.response = req.header.response;
        resp.header.rescode = result_code;
        let mut resp_buffer = dns::BytePacketBuffer::new();
        resp.write(&mut resp_buffer)?;
        Ok((resp, resp_buffer))
    }
}

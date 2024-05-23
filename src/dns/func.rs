use std::net::{Ipv4Addr, UdpSocket};

use super::BytePacketBuffer;
use super::Message;
use super::QueryType;
use super::Question;
use super::Result;

pub fn lookup(
    dns_server: Ipv4Addr,
    id: u16,
    name: impl Into<String>,
    qtype: QueryType,
    class: u16,
) -> Result<Message> {
    let server = (dns_server, 53);
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;
    let mut msg = Message::new();
    msg.header.id = id;
    msg.header.questions = 1;
    msg.header.recursion_desired = true;
    msg.questions.push(Question::new(name.into(), qtype, class));

    let mut req = BytePacketBuffer::new();
    msg.write(&mut req)?;
    socket.send_to(&req.buf[0..req.pos], server)?;

    let mut resp = BytePacketBuffer::new();
    socket.recv_from(&mut resp.buf)?;

    Message::read(&mut resp)
}

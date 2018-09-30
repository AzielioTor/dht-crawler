use serde::Serialize;
use serde::Serializer;

use serde::de;
use serde::Deserialize;
use serde::Deserializer;

use std::net::SocketAddrV4;

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use core::fmt;
use serde::de::Visitor;
use std::net::Ipv4Addr;

/// Wrapper type handling compact serialization and de-serialization of ip address and port
/// information. Defined in BEP5.
#[derive(Eq, PartialEq, Debug)]
pub struct Addr(pub SocketAddrV4);

pub fn write_to(addr: &SocketAddrV4, raw: &mut [u8]) {
    let ip = addr.ip();
    let port = addr.port();

    raw[..4].clone_from_slice(&ip.octets());
    (&mut raw[4..])
        .write_u16::<NetworkEndian>(port)
        .expect("Failed to encode port.");
}

fn to_bytes(addr: &SocketAddrV4) -> [u8; 6] {
    let mut raw = [0u8; 6];
    write_to(addr, &mut raw);

    raw
}

pub fn from_bytes(v: &[u8]) -> SocketAddrV4 {
    let ip = Ipv4Addr::new(v[0], v[1], v[2], v[3]);
    let port = (&v[4..]).read_u16::<NetworkEndian>().unwrap();

    SocketAddrV4::new(ip, port)
}

impl Serialize for Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&to_bytes(&self.0))
    }
}

impl<'de> Deserialize<'de> for Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(NodeInfoVisitor)
    }
}

struct NodeInfoVisitor;

impl<'de> Visitor<'de> for NodeInfoVisitor {
    type Value = Addr;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array of size 6")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let len = v.len();
        if len != 6 {
            return Err(de::Error::invalid_length(len, &self));
        }

        Ok(Addr(from_bytes(v)))
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_test;

    use self::serde_test::{assert_tokens, Token};
    use proto::Addr;
    use std::net::Ipv4Addr;
    use std::net::SocketAddrV4;

    #[test]
    fn serde() {
        let addr = Ipv4Addr::new(129, 21, 60, 66);
        let port = 12019;
        let socket_addr = SocketAddrV4::new(addr, port);

        let node_info = Addr(socket_addr);

        assert_tokens(&node_info, &[Token::Bytes(&[129, 21, 60, 66, 0x2e, 0xf3])]);
    }
}

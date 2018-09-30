use serde::de;
use serde::de::Visitor;
use serde::Deserializer;
use serde::Serializer;

use std::fmt;
use std::net::SocketAddrV4;

use super::{addr, NodeID};

#[derive(Debug, PartialEq, Eq)]
pub struct NodeInfo {
    pub node_id: NodeID,
    pub address: SocketAddrV4,
}

impl NodeInfo {
    pub fn new(node_id: NodeID, addr: SocketAddrV4) -> NodeInfo {
        NodeInfo {
            node_id,
            address: addr,
        }
    }

    fn to_bytes(&self) -> [u8; 26] {
        let mut output = [0u8; 26];
        output.copy_from_slice(&self.node_id.get_bytes()[..]);
        addr::write_to(&self.address, &mut output[20..]);

        output
    }

    fn from_bytes(bytes: &[u8]) -> NodeInfo {
        let node_id = NodeID::from_bytes(&bytes[..20]);
        let address = addr::from_bytes(&bytes[20..]);

        NodeInfo { node_id, address }
    }
}

pub fn serialize<S>(nodes: &Vec<NodeInfo>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bytes = nodes
        .iter()
        .flat_map(|node| node.to_bytes().to_vec())
        .collect::<Vec<u8>>();

    serializer.serialize_bytes(&bytes)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<NodeInfo>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_bytes(NodeInfoVecVisitor)
}

struct NodeInfoVecVisitor;

impl<'de> Visitor<'de> for NodeInfoVecVisitor {
    type Value = Vec<NodeInfo>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array with a size which is a multiple of 26")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let len = v.len();
        if len % 26 != 0 {
            return Err(de::Error::invalid_length(len, &self));
        }

        let mut output: Vec<NodeInfo> = Vec::with_capacity(len / 26);

        for idx in (0..len).step_by(26) {
            let node_info = NodeInfo::from_bytes(&v[idx..]);
            output.push(node_info);
        }

        Ok(output)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(&v)
    }
}
// Smoldot
// Copyright (C) 2019-2021  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

// TODO: work in progress

use crate::libp2p::{multiaddr, peer_id};

use alloc::vec::Vec;

use prost::Message as _;

pub mod kbuckets;

mod dht_proto {
    // File generated by the build script.
    include!(concat!(env!("OUT_DIR"), "/dht.pb.rs"));
}

/// Data structure containing the k-buckets and the state of the current Kademlia queries.
// TODO: unused
pub struct Kademlia {}

impl Kademlia {
    /// Initializes a new empty data structure with empty k-buckets.
    pub fn new() -> Self {
        Kademlia {}
    }
}

impl Default for Kademlia {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a wire message to send on the Kademlia request-response protocol to ask the target to
/// return the nodes closest to the parameter.
// TODO: parameter type?
pub fn build_find_node_request(peer_id: &[u8]) -> Vec<u8> {
    let protobuf = dht_proto::Message {
        r#type: dht_proto::message::MessageType::FindNode as i32,
        key: peer_id.to_vec(),
        ..Default::default()
    };

    let mut buf = Vec::with_capacity(protobuf.encoded_len());
    protobuf.encode(&mut buf).unwrap();
    buf
}

/// Decodes a response to a request built using [`build_find_node_request`].
// TODO: return a borrow of the response bytes ; we're limited by protobuf library
pub fn decode_find_node_response(
    response_bytes: &[u8],
) -> Result<Vec<(peer_id::PeerId, Vec<multiaddr::Multiaddr>)>, DecodeFindNodeResponseError> {
    let response = dht_proto::Message::decode(response_bytes)
        .map_err(ProtobufDecodeError)
        .map_err(DecodeFindNodeResponseError::ProtobufDecode)?;

    if response.r#type != dht_proto::message::MessageType::FindNode as i32 {
        return Err(DecodeFindNodeResponseError::BadResponseTy);
    }

    let mut result = Vec::with_capacity(response.closer_peers.len());
    for peer in response.closer_peers {
        let peer_id = peer_id::PeerId::from_bytes(peer.id)
            .map_err(|(err, _)| DecodeFindNodeResponseError::BadPeerId(err))?;

        let mut multiaddrs = Vec::with_capacity(peer.addrs.len());
        for addr in peer.addrs {
            let addr = multiaddr::Multiaddr::try_from(addr)
                .map_err(DecodeFindNodeResponseError::BadMultiaddr)?;
            multiaddrs.push(addr);
        }

        result.push((peer_id, multiaddrs));
    }

    Ok(result)
}

/// Error potentially returned by [`decode_find_node_response`].
#[derive(Debug, derive_more::Display)]
pub enum DecodeFindNodeResponseError {
    /// Error while decoding the protobuf encoding.
    ProtobufDecode(ProtobufDecodeError),
    /// Response isn't a response to a find node request.
    BadResponseTy,
    /// Error while parsing a [`peer_id::PeerId`] in the response.
    BadPeerId(peer_id::FromBytesError),
    /// Error while parsing a [`multiaddr::Multiaddr`] in the response.
    BadMultiaddr(multiaddr::FromVecError),
}

/// Error while decoding the protobuf encoding.
#[derive(Debug, derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct ProtobufDecodeError(prost::DecodeError);

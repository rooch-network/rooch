// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::state::FieldKey;
use primitive_types::H256;
use smt::jellyfish_merkle::node_type::Node;
use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;

/// Read unsigned LEB128 from the given byte slice.
/// Returns the parsed value and the number of bytes consumed.
pub fn read_uleb128(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut result = 0usize;
    let mut shift = 0;
    for (i, &b) in bytes.iter().enumerate() {
        let value = (b & 0x7F) as usize;
        result |= value << shift;
        if (b & 0x80) == 0 {
            return Some((result, i + 1));
        }
        shift += 7;
        if shift >= usize::BITS as usize {
            // Overflow – malformed input
            return None;
        }
    }
    None
}

/// Attempt to extract an embedded table root hash from a serialized leaf node.
/// The on-disk layout is: `[tag 0x02][key 32B][ULEB128 len][payload len bytes]`.
/// We assume the payload is `[32B child_root][4B entry_count]` when `len == 36`.
pub fn try_extract_child_root(bytes: &[u8]) -> Option<H256> {
    // Decode the SMT node via BCS; only Leaf nodes contain the value we care about.
    let node = Node::<FieldKey, moveos_types::state::ObjectState>::decode(bytes).ok()?;
    if let Node::Leaf(leaf) = node {
        let state = &leaf.value().origin;
        if let Some(hash) = state.metadata.state_root {
            if hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                return Some(hash);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use moveos_types::state::{FieldKey, ObjectState};
    use moveos_types::test_utils::random_table_object;

    fn uleb128(mut v: usize) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let mut byte = (v & 0x7F) as u8;
            v >>= 7;
            if v != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if v == 0 {
                break;
            }
        }
        out
    }

    #[test]
    fn test_extract_child_root_none_variant_none() {
        let mut bytes = Vec::new();
        bytes.push(2u8);
        bytes.extend_from_slice(&[0u8; 32]);
        let payload_len = 98usize;
        bytes.extend_from_slice(&uleb128(payload_len));
        bytes.extend_from_slice(&[0xAAu8; 32]);
        bytes.extend_from_slice(&[0xBBu8; 32]);
        bytes.push(0);
        // Option::None discriminator
        bytes.push(0);
        bytes.extend_from_slice(&[0u8; 32]);

        let extracted = try_extract_child_root(&bytes);
        assert!(extracted.is_none());
    }

    #[test]
    fn test_extract_child_root_some_real_object_state() {
        let table = random_table_object();
        let mut meta = table.metadata().clone();
        let expected_root = H256::random();
        meta.state_root = Some(expected_root);
        let state = ObjectState::new_with_struct(meta, table.value.clone()).unwrap();
        let leaf = smt::jellyfish_merkle::node_type::LeafNode::new(
            FieldKey::new(*H256::random().as_fixed_bytes()),
            smt::SMTObject::from_origin(state).unwrap(),
        );
        let leaf_bytes =
            smt::jellyfish_merkle::node_type::Node::<FieldKey, ObjectState>::Leaf(leaf)
                .encode()
                .unwrap();

        let extracted = try_extract_child_root(&leaf_bytes);
        assert_eq!(extracted, Some(expected_root));
    }
}

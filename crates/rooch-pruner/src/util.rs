// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_types::state::FieldKey;
use primitive_types::H256;
use smt::jellyfish_merkle::node_type::Node;
use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;
use tracing::warn;

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

/// Extract all child node hashes from a serialized SMT node.
///
/// This function handles both Internal nodes and Leaf nodes:
/// - For Internal nodes: returns all child hashes
/// - For Leaf nodes: returns embedded table root if present
///
/// This avoids redundant deserialization by parsing the node only once.
pub fn extract_child_nodes(bytes: &[u8]) -> Vec<H256> {
    match extract_child_nodes_strict(bytes) {
        Ok(children) => children,
        Err(e) => {
            warn!(
                "Failed to decode SMT node (len={}): {}. This may indicate data corruption.",
                bytes.len(),
                e
            );
            Vec::new()
        }
    }
}

pub fn extract_child_nodes_strict(bytes: &[u8]) -> Result<Vec<H256>> {
    let node = Node::<FieldKey, moveos_types::state::ObjectState>::decode(bytes)?;

    let children = match node {
        Node::Internal(internal) => {
            // For internal nodes, return all child hashes
            internal.all_child().into_iter().map(|h| h.into()).collect()
        }
        Node::Leaf(leaf) => {
            // For leaf nodes, check if there's an embedded table root
            let state = &leaf.value().origin;
            if let Some(hash) = state.metadata.state_root {
                if hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                    return Ok(vec![hash]);
                }
            }
            Vec::new()
        }
        Node::Null => Vec::new(),
    };

    Ok(children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use moveos_types::state::{FieldKey, ObjectState};
    use moveos_types::test_utils::random_table_object;
    use smt::jellyfish_merkle::node_type::LeafNode;
    use smt::{InMemoryNodeStore, NodeReader, SMTree, UpdateSet};

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
        let leaf = LeafNode::new(
            FieldKey::new(*H256::random().as_fixed_bytes()),
            smt::SMTObject::from_origin(state).unwrap(),
        );
        let leaf_bytes = Node::<FieldKey, ObjectState>::Leaf(leaf).encode().unwrap();

        let extracted = try_extract_child_root(&leaf_bytes);
        assert_eq!(extracted, Some(expected_root));
    }

    // Tests for extract_child_nodes

    #[test]
    fn test_extract_child_nodes_empty_bytes() {
        let children = extract_child_nodes(&[]);
        assert!(children.is_empty());
    }

    #[test]
    fn test_extract_child_nodes_null_node() {
        // Null node has tag = 0
        let null_bytes = Node::<FieldKey, ObjectState>::new_null().encode().unwrap();
        let children = extract_child_nodes(&null_bytes);
        assert!(children.is_empty());
    }

    #[test]
    fn test_extract_child_nodes_internal_node() {
        // Create an SMT and insert multiple keys to generate an internal node
        let node_store = InMemoryNodeStore::default();
        let registry = prometheus::Registry::new();
        let smt = SMTree::<FieldKey, ObjectState, _>::new(node_store.clone(), &registry);

        // Create multiple distinct objects to insert
        let mut updates = UpdateSet::new();
        for i in 0..10 {
            let table = random_table_object();
            let mut meta = table.metadata().clone();
            // Use different hashes to force tree structure
            let key_bytes = {
                let mut bytes = [0u8; 32];
                bytes[0] = i;
                bytes[31] = i;
                bytes
            };
            meta.state_root = None;
            let state = ObjectState::new_with_struct(meta, table.value.clone()).unwrap();
            updates.put(FieldKey::new(key_bytes), state);
        }

        // Put the updates into the tree
        let changeset = smt.puts(*SPARSE_MERKLE_PLACEHOLDER_HASH, updates).unwrap();
        node_store.write_nodes(changeset.nodes).unwrap();

        // The root should be an internal node since we have multiple entries
        let root_bytes = node_store.get(&changeset.state_root).unwrap().unwrap();

        // Verify the root is an internal node (tag = 1)
        assert_eq!(root_bytes[0], 1, "Root should be an internal node");

        let children = extract_child_nodes(&root_bytes);
        // Internal node should have children
        assert!(!children.is_empty(), "Internal node should have children");
    }

    #[test]
    fn test_extract_child_nodes_leaf_with_child_root() {
        let table = random_table_object();
        let mut meta = table.metadata().clone();
        let expected_root = H256::random();
        meta.state_root = Some(expected_root);
        let state = ObjectState::new_with_struct(meta, table.value.clone()).unwrap();
        let leaf = LeafNode::new(
            FieldKey::new(*H256::random().as_fixed_bytes()),
            smt::SMTObject::from_origin(state).unwrap(),
        );
        let leaf_bytes = Node::<FieldKey, ObjectState>::Leaf(leaf).encode().unwrap();

        let children = extract_child_nodes(&leaf_bytes);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], expected_root);
    }

    #[test]
    fn test_extract_child_nodes_leaf_without_child_root() {
        let table = random_table_object();
        let mut meta = table.metadata().clone();
        // No state_root set (None)
        meta.state_root = None;
        let state = ObjectState::new_with_struct(meta, table.value.clone()).unwrap();
        let leaf = LeafNode::new(
            FieldKey::new(*H256::random().as_fixed_bytes()),
            smt::SMTObject::from_origin(state).unwrap(),
        );
        let leaf_bytes = Node::<FieldKey, ObjectState>::Leaf(leaf).encode().unwrap();

        let children = extract_child_nodes(&leaf_bytes);
        assert!(children.is_empty());
    }

    #[test]
    fn test_extract_child_nodes_leaf_with_placeholder_root() {
        let table = random_table_object();
        let mut meta = table.metadata().clone();
        // Set to placeholder hash (should be ignored)
        meta.state_root = Some(*SPARSE_MERKLE_PLACEHOLDER_HASH);
        let state = ObjectState::new_with_struct(meta, table.value.clone()).unwrap();
        let leaf = LeafNode::new(
            FieldKey::new(*H256::random().as_fixed_bytes()),
            smt::SMTObject::from_origin(state).unwrap(),
        );
        let leaf_bytes = Node::<FieldKey, ObjectState>::Leaf(leaf).encode().unwrap();

        let children = extract_child_nodes(&leaf_bytes);
        assert!(children.is_empty());
    }

    #[test]
    fn test_extract_child_nodes_unknown_tag() {
        // Unknown tag (e.g., 255)
        let children = extract_child_nodes(&[255, 0, 0, 0]);
        assert!(children.is_empty());
    }
}

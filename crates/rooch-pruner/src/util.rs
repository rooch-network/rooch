// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;

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
    // Minimal length check: tag (1) + key (32) + len (1) + payload (36)
    if bytes.len() < 70 {
        return None;
    }

    if bytes[0] != 2 {
        // Not a leaf node
        return None;
    }

    // Skip tag and key
    let mut pos = 1 + 32;

    // Parse ULEB128 length
    let (len, leb_len) = read_uleb128(&bytes[pos..])?;
    pos += leb_len;

    // We only care about the special 36-byte payload case
    if len != 36 || pos + 36 > bytes.len() {
        return None;
    }

    // Take the first 32 bytes as the child root hash
    Some(H256::from_slice(&bytes[pos..pos + 32]))
}

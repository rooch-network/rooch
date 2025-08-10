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
    // Ensure node tag indicates Leaf
    if bytes.first()? != &2u8 {
        return None;
    }

    // Skip tag (1) + key (32)
    let mut pos = 1 + 32;

    // Read payload length
    let (len, leb_len) = read_uleb128(&bytes[pos..])?;
    pos += leb_len;
    if pos + len > bytes.len() {
        return None;
    }
    let payload = &bytes[pos..pos + len];

    // Fast-parse ObjectState metadata without full BCS
    let mut p = 0;
    // 1. ObjectID (32 bytes)
    if payload.len() < p + 32 {
        return None;
    }
    p += 32;
    // 2. owner AccountAddress (32 bytes)
    if payload.len() < p + 32 {
        return None;
    }
    p += 32;
    // 3. flag (1 byte)
    if payload.len() < p + 1 {
        return None;
    }
    p += 1;

    // 4. Option<H256> variant for state_root
    if payload.len() <= p {
        return None;
    }
    let variant = payload[p];
    p += 1;
    if variant != 1 {
        // Option::None => not a table root
        return None;
    }
    if payload.len() < p + 32 {
        return None;
    }
    let hash = H256::from_slice(&payload[p..p + 32]);

    // Skip size (u64) + created_at (u64) + updated_at (u64)
    if payload.len() < p + 24 {
        return None;
    }
    p += 24;

    // Parse TypeTag to ensure it is TablePlaceholder
    if payload.len() <= p {
        return None;
    }
    let tt_variant = payload[p]; // 4 == Struct
    p += 1;
    if tt_variant != 4 {
        // not Struct TypeTag
        return None;
    }

    // Skip address (32 bytes)
    if payload.len() < p + 32 {
        return None;
    }
    p += 32;

    // Read module name
    let (mod_len, mod_len_bytes) = read_uleb128(&payload[p..])?;
    p += mod_len_bytes;
    if payload.len() < p + mod_len {
        return None;
    }
    let mod_name = &payload[p..p + mod_len];
    p += mod_len;

    // Read struct name
    let (struct_len, struct_len_bytes) = read_uleb128(&payload[p..])?;
    p += struct_len_bytes;
    if payload.len() < p + struct_len {
        return None;
    }
    let struct_name = &payload[p..p + struct_len];

    if mod_name == b"table" && struct_name == b"TablePlaceholder" {
        return Some(hash);
    }
    None
}

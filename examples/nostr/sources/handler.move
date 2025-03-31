// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 for handling client requests
module nostr::handlers {
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use nostr::inner::Tags;
    /// TODO: NIP-01: index the a single alphabet letter tag with the first value returned to be used with tag filter from the client
    fun index_tags(tags: vector<Tags>): SimpleMultiMap<String, String> {
        // create a simple multi map for the single-letter english alphabet letters of tag index
        let alphabet = simple_multimap::new<String, String>();
    }
}

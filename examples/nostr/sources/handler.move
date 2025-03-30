// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 for handling client requests
module nostr::handlers {
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use nostr::inner::{Self, EventData};
    /// NIP-01: index the a single alphabet letter tag with the first value returned to be used with tag filter from the client
    fun index_single_alphabet_letter_tag(event_data: EventData): SimpleMultiMap<String, String> {

    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::ability {
    use std::string;

    const ABILITY_COPY:u8 = 0x1;
    public fun ability_copy() : u8 {
        ABILITY_COPY
    }

    const ABILITY_DROP:u8 = 0x2;
    public fun ability_drop() : u8 {
        ABILITY_DROP
    }

    const ABILITY_STORE:u8 = 0x4;
    public fun ability_store() : u8 {
        ABILITY_STORE
    }

    const ABILITY_KEY:u8 = 0x8;
    public fun ability_key() : u8 {
        ABILITY_KEY
    }
    public fun has_ability(abilities: u8, ability: u8): bool {
        (ability & abilities) == ability
    }

    public fun has_copy(abilities: u8): bool {
        Self::has_ability(abilities, ability_copy())
    }

    public fun has_drop(abilities: u8): bool {
        Self::has_ability(abilities, ability_drop())
    }

    public fun has_store(abilities: u8): bool {
        Self::has_ability(abilities, ability_store())
    }

    public fun has_key(abilities: u8): bool {
        Self::has_ability(abilities, ability_key())
    }

    native public fun native_get_abilities(type: string::String): u8;
}
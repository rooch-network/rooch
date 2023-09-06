// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ```
//! use hex_literal::hex;
//! use ethbloom::{Bloom, Input};
//!
//! use std::str::FromStr;
//! let bloom = Bloom::from_str(
//! 	"00000000000000000000000000000000\
//! 	00000000100000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000002020000000000000000000000\
//! 	00000000000000000000000800000000\
//! 	10000000000000000000000000000000\
//! 	00000000000000000000001000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000\
//! 	00000000000000000000000000000000"
//! ).unwrap();
//! let address = hex!("ef2d6d194084c2de36e0dabfce45d046b37d1106");
//! let topic = hex!("02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc");
//!
//! let mut my_bloom = Bloom::default();
//! assert!(!my_bloom.contains_input(Input::Raw(&address)));
//! assert!(!my_bloom.contains_input(Input::Raw(&topic)));
//!
//! my_bloom.accrue(Input::Raw(&address));
//! assert!(my_bloom.contains_input(Input::Raw(&address)));
//! assert!(!my_bloom.contains_input(Input::Raw(&topic)));
//!
//! my_bloom.accrue(Input::Raw(&topic));
//! assert!(my_bloom.contains_input(Input::Raw(&address)));
//! assert!(my_bloom.contains_input(Input::Raw(&topic)));
//! assert_eq!(my_bloom, bloom);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

use core::{mem, ops};

use fixed_hash::*;
#[cfg(feature = "codec")]
use impl_codec::impl_fixed_hash_codec;
#[cfg(feature = "rlp")]
use impl_rlp::impl_fixed_hash_rlp;
#[cfg(feature = "serialize")]
use impl_serde::impl_fixed_hash_serde;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

// 3 according to yellowpaper
// const BLOOM_BITS: u32 = 3;
// const BLOOM_SIZE: usize = 256;

construct_fixed_hash! {
    /// Bloom hash type with 256 bytes (2048 bits) size.
	#[cfg_attr(feature = "codec", derive(scale_info::TypeInfo))]
	#[derive(Serialize, Deserialize, JsonSchema)]
	#[serde(rename_all = "camelCase")]
	pub struct Bloom(32); // TODO change to 256
}
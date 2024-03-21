// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: CC0-1.0

//! Bitcoin hash types.
//!
//! This module defines types for hashes used throughout the library. These
//! types are needed in order to avoid mixing data of the same hash format
//! (e.g. `SHA256d`) but of different meaning (such as transaction id, block
//! hash).
//!

#[rustfmt::skip]
macro_rules! impl_hashencode {
    ($hashtype:ident) => {
        impl bitcoin::consensus::Encodable for $hashtype {
            fn consensus_encode<W: std::io::Write + ?Sized>(&self, w: &mut W) -> Result<usize, std::io::Error> {
                self.0.consensus_encode(w)
            }
        }

        impl bitcoin::consensus::Decodable for $hashtype {
            fn consensus_decode<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, bitcoin::consensus::encode::Error> {
                use bitcoin::hashes::Hash;
                Ok(Self::from_byte_array(<<$hashtype as bitcoin::hashes::Hash>::Bytes>::consensus_decode(r)?))
            }
        }
    };
}

// newtypes module is solely here so we can rustfmt::skip.
pub use newtypes::*;

#[rustfmt::skip]
mod newtypes {
    use bitcoin::hashes;
    use hashes::hash_newtype;
    use schemars::JsonSchema;

    use crate::jsonrpc_types::ordinal::bitcoin_hashes::SHA256DView;

    hash_newtype! {
        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// A bitcoin transaction hash/transaction ID.
        ///
        /// For compatibility with the existing Bitcoin infrastructure and historical
        /// and current versions of the Bitcoin Core software itself, this and
        /// other [`SHA256DView`] types, are serialized in reverse
        /// byte order when converted to a hex string via [`std::fmt::Display`] trait operations.
        /// See [`hashes::Hash::DISPLAY_BACKWARD`] for more details.
        pub struct TxidView(SHA256DView); 

        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// A bitcoin witness transaction ID.
        pub struct WtxidView(SHA256DView);
        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// A bitcoin block hash.
        pub struct BlockHashView(SHA256DView);

        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// A hash of the Merkle tree branch or root for transactions
        pub struct TxMerkleNodeView(SHA256DView);
        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// A hash corresponding to the Merkle tree root for witness data
        pub struct WitnessMerkleNodeView(SHA256DView);
        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// A hash corresponding to the witness structure commitment in the coinbase transaction
        pub struct WitnessCommitmentView(SHA256DView);

        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// Filter hash, as defined in BIP-157
        pub struct FilterHashView(SHA256DView);
        #[derive(JsonSchema)]
        #[serde(rename_all = "lowercase")]
        /// Filter header, as defined in BIP-157
        pub struct FilterHeaderView(SHA256DView);
    }

    impl_hashencode!(TxidView);
    impl_hashencode!(WtxidView);
    impl_hashencode!(BlockHashView);

    impl_hashencode!(TxMerkleNodeView);
    impl_hashencode!(WitnessMerkleNodeView);

    impl_hashencode!(FilterHashView);
    impl_hashencode!(FilterHeaderView);
}

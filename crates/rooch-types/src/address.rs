// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::ROOCH_FRAMEWORK_ADDRESS,
    chain_id::{ChainID, CustomChainID, RoochChainID},
};
use anyhow::Result;
use bech32::{FromBase32, ToBase32};
use bitcoin::{
    address::{Address, Payload, WitnessProgram, WitnessVersion},
    hashes::{hash160, Hash},
    network::constants::Network,
};
use ethers::types::H160;
use fastcrypto::secp256k1::recoverable::Secp256k1RecoverablePublicKey;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
#[cfg(any(test, feature = "fuzzing"))]
use moveos_types::h256;
use moveos_types::{
    h256::H256,
    state::{MoveStructState, MoveStructType},
};
use nostr::secp256k1::XOnlyPublicKey;
use nostr::Keys;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use sha3::{Digest, Sha3_256};
use std::fmt;
use std::str::FromStr;

/// The address type that Rooch supports
pub trait RoochSupportedAddress:
    Into<MultiChainAddress> + TryFrom<MultiChainAddress, Error = anyhow::Error>
{
    fn random() -> Self;
}

/// Multi chain address representation
/// The address is distinguished by the multichain id type, multichain id type standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct MultiChainAddress {
    pub multichain_id: RoochChainID,
    pub raw_address: Vec<u8>,
}

impl MultiChainAddress {
    pub(crate) fn new(multichain_id: RoochChainID, raw_address: Vec<u8>) -> Result<Self> {
        Ok(Self {
            multichain_id,
            raw_address,
        })
    }

    pub fn is_rooch_address(&self) -> bool {
        self.multichain_id.is_builtin()
    }

    pub fn from_bech32(bech32: &str) -> Result<Self> {
        let (hrp, data, variant) = bech32::decode(bech32)?;
        if variant != bech32::Variant::Bech32 {
            return Err(anyhow::anyhow!("invalid bech32 variant"));
        }
        let version = data.first().map(|u| u.to_u8());
        anyhow::ensure!(version.filter(|v| *v == 1u8).is_some(), "expect version 1");

        let multichain_id = RoochChainID::from_str(hrp.as_str())?;
        let address = Vec::<u8>::from_base32(&data[1..])?;
        Ok(Self {
            multichain_id,
            raw_address: address,
        })
    }

    pub fn to_bech32(&self) -> String {
        let mut data = self.raw_address.to_base32();
        //A Bech32 string consists of a human-readable part (HRP), a separator (the character '1'), and a data part
        data.insert(
            0,
            bech32::u5::try_from_u8(1).expect("1 to u8 should success"),
        );
        bech32::encode(
            &self.multichain_id.to_string().to_lowercase(),
            data,
            bech32::Variant::Bech32,
        )
        .expect("bech32 encode should success")
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("bcs encode should success")
    }
}

impl Serialize for MultiChainAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_bech32())
        } else {
            #[derive(::serde::Serialize)]
            #[serde(rename = "MultiChainAddress")]
            struct Value(u64, Vec<u8>);
            let value = Value(self.multichain_id.chain_id().id(), self.raw_address.clone());
            value.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for MultiChainAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let bech32 = String::deserialize(deserializer)?;
            Self::from_bech32(&bech32).map_err(serde::de::Error::custom)
        } else {
            #[derive(::serde::Deserialize)]
            #[serde(rename = "MultiChainAddress")]
            struct Value(u64, Vec<u8>);
            let value = Value::deserialize(deserializer)?;
            Self::new(
                RoochChainID::try_from(ChainID::from(value.0)).map_err(serde::de::Error::custom)?,
                value.1,
            )
            .map_err(serde::de::Error::custom)
        }
    }
}

//TODO do not use bech32 to represent address
//Use multichain_id:original_address to represent multichain_id address,
//eth:0x1234.., btc:1px99y..., roh:0x1234..
impl std::fmt::Display for MultiChainAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_bech32())
    }
}

impl FromStr for MultiChainAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bech32(s)
    }
}

impl MoveStructType for MultiChainAddress {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("address_mapping");
    const STRUCT_NAME: &'static IdentStr = ident_str!("MultiChainAddress");
}

impl MoveStructState for MultiChainAddress {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U32,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }
}
/// Rooch address type
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct RoochAddress(pub H256);

impl RoochSupportedAddress for RoochAddress {
    fn random() -> Self {
        Self(H256::random())
    }
}

impl From<AccountAddress> for RoochAddress {
    fn from(address: AccountAddress) -> Self {
        Self(H256(address.into()))
    }
}

impl From<RoochAddress> for AccountAddress {
    fn from(address: RoochAddress) -> Self {
        AccountAddress::from(address.0 .0)
    }
}

impl From<RoochAddress> for MultiChainAddress {
    fn from(address: RoochAddress) -> Self {
        Self::new(RoochChainID::DEV, address.0.as_bytes().to_vec())
            .expect("RoochAddress to MultiChainAddress should success")
    }
}

impl TryFrom<MultiChainAddress> for RoochAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochChainID::DEV {
            return Err(anyhow::anyhow!(
                "multichain_id type {} is invalid",
                value.multichain_id
            ));
        }
        Ok(Self(H256::from_slice(&value.raw_address)))
    }
}

// ==== Display and FromStr, Deserialize and Serialize ====

impl fmt::Display for RoochAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        //Use full address and 0x prefix for address display
        write!(f, "0x{}", AccountAddress::from(*self))
    }
}

impl fmt::Debug for RoochAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", AccountAddress::from(*self).to_hex_literal())
    }
}

impl FromStr for RoochAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = AccountAddress::from_str(s)?;
        Ok(Self::from(address))
    }
}

impl<'de> Deserialize<'de> for RoochAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            Self::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            let value = AccountAddress::deserialize(deserializer)?;
            Ok(value.into())
        }
    }
}

impl Serialize for RoochAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            let account_address: AccountAddress = (*self).into();
            account_address.serialize(serializer)
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for RoochAddress {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_rooch_address().boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_rooch_address()(
     bytes in vec(any::<u8>(), h256::LENGTH..(h256::LENGTH+1))
    ) -> RoochAddress {
        RoochAddress(H256::from_slice(&bytes))
    }
}

/// Ethereum address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd, Copy)]
#[serde_as]
pub struct EthereumAddress(pub H160);

impl fmt::Display for EthereumAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the Ethereum address as a hexadecimal string with a "0x" prefix
        write!(f, "0x{}", self.0)
    }
}

impl RoochSupportedAddress for EthereumAddress {
    fn random() -> Self {
        Self(H160::random())
    }
}

impl From<EthereumAddress> for MultiChainAddress {
    fn from(address: EthereumAddress) -> Self {
        Self::new(
            RoochChainID::from(CustomChainID::ethereum()),
            address.0.as_bytes().to_vec(),
        )
        .expect("EthereumAddress to MultiChainAddress should success")
    }
}

impl TryFrom<MultiChainAddress> for EthereumAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochChainID::from(CustomChainID::ethereum()) {
            return Err(anyhow::anyhow!(
                "multichain_id type {} is invalid",
                value.multichain_id
            ));
        }
        Ok(Self(H160::from_slice(&value.raw_address)))
    }
}

impl From<Secp256k1RecoverablePublicKey> for EthereumAddress {
    fn from(value: Secp256k1RecoverablePublicKey) -> Self {
        // Take uncompressed public key
        let uncompressed_public_key_bytes = value.pubkey.serialize_uncompressed();
        // Ignore the first byte and take the last 64-bytes of the uncompressed pubkey
        let uncompressed_64 = uncompressed_public_key_bytes[1..].to_vec();
        // create a SHA3-256 object
        let mut hasher = Sha3_256::new();
        // write input message
        hasher.update(&uncompressed_64);
        // read hash digest
        let result = hasher.finalize();
        // Take the last 20 bytes of the hash of the 64-bytes uncompressed pubkey
        let address_bytes = result[12..32].to_vec();
        Self(H160::from_slice(&address_bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitcoinAddress(pub Address);

impl RoochSupportedAddress for BitcoinAddress {
    fn random() -> Self {
        // Generate a random public key hash
        let pubkey_hash = hash160::Hash::from_slice(H160::random().as_bytes()).unwrap();
        // Create a P2PKH address using the public key hash
        let p2pkh_address = Address::new(Network::Bitcoin, Payload::PubkeyHash(pubkey_hash.into()));
        // Create a redeem script from the P2PKH address
        let redeem_script = p2pkh_address.script_pubkey();
        // Create a P2SH address using the redeem script
        let p2sh_address = Address::new(
            Network::Bitcoin,
            Payload::ScriptHash(redeem_script.script_hash()),
        );
        // Create a witness program for the SegWit address
        let witness_program = vec![0x00]
            .into_iter()
            .chain(pubkey_hash.as_byte_array().to_vec())
            .collect::<Vec<u8>>();
        // Create a SegWit address using the witness program
        let segwit_address = Address::new(
            Network::Bitcoin,
            Payload::WitnessProgram(
                WitnessProgram::new(WitnessVersion::V1, witness_program.to_vec()).unwrap(),
            ),
        );
        // Create an array of addresses bitcoin protocols
        let addresses = [p2pkh_address, p2sh_address, segwit_address];
        // Randomly select one of the addresses
        let mut rng = thread_rng();
        let selected_address = addresses.choose(&mut rng).unwrap().clone();
        // Return the randomly selected Bitcoin address
        BitcoinAddress(selected_address)
    }
}

impl From<BitcoinAddress> for MultiChainAddress {
    fn from(address: BitcoinAddress) -> Self {
        Self::new(
            RoochChainID::from(CustomChainID::bitcoin()),
            address.0.to_string().into_bytes(),
        )
        .expect("BitcoinAddress to MultiChainAddress should succeed")
    }
}

impl TryFrom<MultiChainAddress> for BitcoinAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochChainID::from(CustomChainID::bitcoin()) {
            return Err(anyhow::anyhow!(
                "multichain_id type {} is invalid",
                value.multichain_id
            ));
        }

        let addr = Address::from_str(&String::from_utf8(value.raw_address)?)
            .map_err(|e| anyhow::anyhow!("invalid bitcoin address: {}", e))?;

        Ok(Self(addr.require_network(bitcoin::Network::Bitcoin)?))
    }
}

/// Nostr address type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NostrAddress(pub XOnlyPublicKey);

impl RoochSupportedAddress for NostrAddress {
    fn random() -> Self {
        Self(Keys::generate().public_key())
    }
}

impl From<NostrAddress> for MultiChainAddress {
    fn from(address: NostrAddress) -> Self {
        Self::new(
            RoochChainID::from(CustomChainID::nostr()),
            address.0.serialize().to_vec(),
        )
        .expect("NostrAddress to MultiChainAddress should succeed")
    }
}

impl TryFrom<MultiChainAddress> for NostrAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochChainID::from(CustomChainID::nostr()) {
            return Err(anyhow::anyhow!(
                "multichain_id type {} is invalid",
                value.multichain_id
            ));
        }
        let addr = XOnlyPublicKey::from_slice(&value.raw_address)?;
        Ok(Self(addr))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn test_rooch_supported_address_roundtrip<T>()
    where
        T: RoochSupportedAddress + Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash,
    {
        let address = T::random();
        println!("{:?}", address);
        let multi_chain_address: MultiChainAddress = address.clone().into();
        let address2 = T::try_from(multi_chain_address.clone()).unwrap();
        assert_eq!(address, address2);
        let addr_str = multi_chain_address.to_string();
        println!("{}", addr_str);
        let address3 = MultiChainAddress::from_str(&addr_str).unwrap();
        assert_eq!(multi_chain_address, address3);
        let address4 = T::try_from(address3).unwrap();
        assert_eq!(address, address4);
    }

    #[test]
    fn test_address() {
        test_rooch_supported_address_roundtrip::<RoochAddress>();
        test_rooch_supported_address_roundtrip::<EthereumAddress>();
        test_rooch_supported_address_roundtrip::<BitcoinAddress>();
        test_rooch_supported_address_roundtrip::<NostrAddress>();
    }

    fn test_rooch_address_roundtrip(rooch_address: RoochAddress) {
        let rooch_str = rooch_address.to_string();
        //ensure the rooch to string is hex with 0x prefix
        //and is full 32 bytes output
        assert!(rooch_str.starts_with("0x"));
        assert_eq!(rooch_str.len(), 66);
        let rooch_address_from_str = RoochAddress::from_str(&rooch_str).unwrap();
        assert_eq!(rooch_address, rooch_address_from_str);

        let json_str = serde_json::to_string(&rooch_address).unwrap();
        assert_eq!(format!("\"{}\"", rooch_str), json_str);
        let rooch_address_from_json: RoochAddress = serde_json::from_str(&json_str).unwrap();
        assert_eq!(rooch_address, rooch_address_from_json);

        let bytes = bcs::to_bytes(&rooch_address).unwrap();
        assert!(bytes.len() == 32);
        let rooch_address_from_bytes = bcs::from_bytes(&bytes).unwrap();
        assert_eq!(rooch_address, rooch_address_from_bytes);
    }

    #[test]
    fn test_rooch_address_to_string() {
        test_rooch_address_roundtrip(RoochAddress::from(AccountAddress::ZERO));
        test_rooch_address_roundtrip(RoochAddress::from(AccountAddress::ONE));
        test_rooch_address_roundtrip(RoochAddress::random());
    }

    proptest! {
        #[test]
        fn test_rooch_address_serialize_deserialize(address in any::<RoochAddress>()) {
            let serialized = serde_json::to_string(&address).unwrap();
            let deserialized: RoochAddress = serde_json::from_str(&serialized).unwrap();
            assert_eq!(address, deserialized);
        }

        #[test]
        fn test_multichain_address_serialize_deserialize(address in any::<MultiChainAddress>()) {
            let serialized = serde_json::to_string(&address).unwrap();
            let deserialized: MultiChainAddress = serde_json::from_str(&serialized).unwrap();
            assert_eq!(address, deserialized);
        }
    }
}

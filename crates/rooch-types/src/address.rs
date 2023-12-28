// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::ROOCH_FRAMEWORK_ADDRESS,
    multichain_id::{MultiChainID, RoochMultiChainID},
};
use anyhow::Result;
use bech32::{FromBase32, ToBase32};
use bitcoin::{address::Address, secp256k1::Secp256k1, Network, PrivateKey, Script};
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
    pub multichain_id: RoochMultiChainID,
    pub raw_address: Vec<u8>,
}

impl MultiChainAddress {
    pub(crate) fn new(multichain_id: RoochMultiChainID, raw_address: Vec<u8>) -> Self {
        Self {
            multichain_id,
            raw_address,
        }
    }

    pub fn is_rooch_address(&self) -> bool {
        self.multichain_id.is_rooch()
    }

    pub fn from_bech32(bech32: &str) -> Result<Self> {
        let (hrp, data, variant) = bech32::decode(bech32)?;
        if variant != bech32::Variant::Bech32 {
            return Err(anyhow::anyhow!("invalid bech32 variant"));
        }
        let version = data.first().map(|u| u.to_u8());
        anyhow::ensure!(version.filter(|v| *v == 1u8).is_some(), "expect version 1");

        let multichain_id = RoochMultiChainID::from_str(hrp.as_str())?;
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
            let value = Value(
                self.multichain_id.multichain_id().id(),
                self.raw_address.clone(),
            );
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
            Ok(Self::new(
                RoochMultiChainID::try_from(MultiChainID::from(value.0))
                    .map_err(serde::de::Error::custom)?,
                value.1,
            ))
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
        Self::new(RoochMultiChainID::Rooch, address.0.as_bytes().to_vec())
    }
}

impl TryFrom<MultiChainAddress> for RoochAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochMultiChainID::Rooch {
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
        Self::new(RoochMultiChainID::Ether, address.0.as_bytes().to_vec())
    }
}

impl TryFrom<MultiChainAddress> for EthereumAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochMultiChainID::Ether {
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

impl FromStr for EthereumAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = H160::from_str(s)?;
        Ok(Self(address))
    }
}

impl From<EthereumAddress> for H160 {
    fn from(address: EthereumAddress) -> Self {
        H160::from(address.0 .0)
    }
}

impl From<H160> for EthereumAddress {
    fn from(h160: H160) -> Self {
        Self(h160)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BitcoinAddress {
    bytes: Vec<u8>,
}

impl Default for BitcoinAddress {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl BitcoinAddress {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn new_p2pkh(pubkey_hash: &bitcoin::PubkeyHash) -> Self {
        let mut bytes = [0; 21];
        //we always use mainnet prefix, do not distinguish testnet and mainnet in Move contract
        bytes[0] = bitcoin::constants::PUBKEY_ADDRESS_PREFIX_MAIN;
        bytes[1..].copy_from_slice(&pubkey_hash[..]);
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn new_p2sh(script_hash: &bitcoin::ScriptHash) -> Self {
        let mut bytes = [0; 21];
        //we always use mainnet prefix, do not distinguish testnet and mainnet in Move contract
        bytes[0] = bitcoin::constants::SCRIPT_ADDRESS_PREFIX_MAIN;
        bytes[1..].copy_from_slice(&script_hash[..]);
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn new_witness_program(witness_program: &bitcoin::WitnessProgram) -> Self {
        let mut bytes = vec![witness_program.version().to_num()];
        bytes.extend_from_slice(witness_program.program().as_bytes());
        Self { bytes }
    }

    /// The empty address is used to if we parse the address failed from the script
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl MoveStructType for BitcoinAddress {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("bitcoin_address");
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinAddress");
}

impl MoveStructState for BitcoinAddress {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8))])
    }
}

impl RoochSupportedAddress for BitcoinAddress {
    fn random() -> Self {
        let secp = Secp256k1::new();
        let p2pkh_address = Address::p2pkh(
            &PrivateKey::generate(Network::Bitcoin).public_key(&secp),
            Network::Bitcoin,
        );
        let p2sh_address = Address::p2sh(
            Script::from_bytes(H160::random().as_bytes()),
            Network::Bitcoin,
        )
        .unwrap();
        let segwit_address = Address::p2wpkh(
            &PrivateKey::generate(Network::Bitcoin).public_key(&secp),
            Network::Bitcoin,
        )
        .unwrap();

        // Create an array of addresses bitcoin protocols
        let addresses = [p2pkh_address, p2sh_address, segwit_address];
        // Randomly select one of the addresses
        let mut rng = thread_rng();
        let selected_address = addresses.choose(&mut rng).unwrap().clone();
        // Return the randomly selected Bitcoin address
        selected_address.into()
    }
}

impl From<bitcoin::Address> for BitcoinAddress {
    fn from(address: bitcoin::Address) -> Self {
        address.payload().into()
    }
}

impl From<&bitcoin::address::Payload> for BitcoinAddress {
    fn from(payload: &bitcoin::address::Payload) -> Self {
        match payload {
            bitcoin::address::Payload::PubkeyHash(pubkey_hash) => Self::new_p2pkh(pubkey_hash),
            bitcoin::address::Payload::ScriptHash(bytes) => Self::new_p2sh(bytes),
            bitcoin::address::Payload::WitnessProgram(program) => {
                Self::new_witness_program(program)
            }
            _ => BitcoinAddress::default(),
        }
    }
}

impl From<bitcoin::address::Payload> for BitcoinAddress {
    fn from(payload: bitcoin::address::Payload) -> Self {
        Self::from(&payload)
    }
}

impl From<BitcoinAddress> for MultiChainAddress {
    fn from(address: BitcoinAddress) -> Self {
        Self::new(RoochMultiChainID::Bitcoin, address.bytes)
    }
}

impl TryFrom<MultiChainAddress> for BitcoinAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochMultiChainID::Bitcoin {
            return Err(anyhow::anyhow!(
                "multichain_id type {} is invalid",
                value.multichain_id
            ));
        }
        Ok(Self::new(value.raw_address))
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
        Self::new(RoochMultiChainID::Nostr, address.0.serialize().to_vec())
    }
}

impl TryFrom<MultiChainAddress> for NostrAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochMultiChainID::Nostr {
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

    fn test_bitcoin_str_address_2_multi_chain_address_roundtrip(address_str: &str) {
        println!("str {:?}", address_str);
        let address = bitcoin::Address::from_str(address_str)
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();
        println!("{:?}", address);
        let bitcoinAddress = BitcoinAddress::from(address);
        println!("{:?}", bitcoinAddress);
        let multiChainAddress = MultiChainAddress::from(bitcoinAddress);
        println!("{:?}", multiChainAddress);
        println!("{}", multiChainAddress);

        let m1 = MultiChainAddress::from_str("bitcoin1pqrawtj0v3gpl4qqkkcavjdequnj9nswvqu40gx7m");
        println!("{:?}", m1)
    }
    #[test]
    fn test_sdk_multi_chain_address() {
        let js_generate_multi_chain_address = "";
        let addressStr = "bc1qltjunmy2q0agq94k8tynwg8yu3vurnq8h7yc7p";

        test_bitcoin_str_address_2_multi_chain_address_roundtrip(addressStr)
    }

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

    #[test]
    pub fn test_from_script() {
        let bytes = hex::decode("001497cdff4fd3ed6f885d54a52b79d7a2141072ae3f").unwrap();
        let script = Script::from_bytes(bytes.as_slice());
        let address = Address::from_script(script, Network::Signet).unwrap();
        //println!("{:?}", address.address_type());
        assert_eq!(
            address.address_type().unwrap(),
            bitcoin::AddressType::P2wpkh
        );
        //println!("{}", address_str);
        assert_eq!(
            address.to_string(),
            "tb1qjlxl7n7na4hcsh25554hn4azzsg89t3ljdldnj"
        )
    }
}

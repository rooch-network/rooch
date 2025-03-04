// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::bitcoin::network;
use crate::crypto::RoochKeyPair;
use crate::to_bech32::{FromBech32, ToBech32, PREFIX_BECH32_PUBLIC_KEY};
use crate::{
    addresses::ROOCH_FRAMEWORK_ADDRESS,
    multichain_id::{MultiChainID, RoochMultiChainID},
};
use anyhow::{bail, Result};
use bech32::segwit::encode_to_fmt_unchecked;
use bech32::Bech32m;
use bitcoin::address::AddressData;
use bitcoin::hashes::Hash;
use bitcoin::params::Params;
use bitcoin::{
    address::Address, secp256k1::Secp256k1, Network, PrivateKey, Script, WitnessProgram,
    WitnessVersion,
};
use bitcoin::{CompressedPublicKey, XOnlyPublicKey};
use ethers::types::H160;
use fastcrypto::hash::Blake2b256;
use fastcrypto::hash::HashFunction;
use fastcrypto::secp256k1::Secp256k1PublicKey;
use hex::FromHex;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use moveos_types::addresses::ROOCH_HRP;
#[cfg(any(test, feature = "fuzzing"))]
use moveos_types::h256;
use moveos_types::state::MoveState;
use moveos_types::{
    h256::H256,
    state::{MoveStructState, MoveStructType},
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
use rand::{seq::SliceRandom, thread_rng};
use serde::ser::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use sha3::{Digest, Sha3_256};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

/// The address type that Rooch supports
pub trait RoochSupportedAddress:
    Into<MultiChainAddress> + TryFrom<MultiChainAddress, Error = anyhow::Error>
{
    fn random() -> Self;
}

/// Multi chain address representation
/// The address is distinguished by the multichain id type, multichain id type standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// The str is the chain original address, such as 0x1234.., 1px99y..., 0x1234..
    pub fn try_from_str_with_multichain_id(
        multichain_id: RoochMultiChainID,
        str: &str,
    ) -> Result<Self, anyhow::Error> {
        match multichain_id {
            RoochMultiChainID::Bitcoin => {
                let address = BitcoinAddress::from_str(str)?;
                Ok(address.into())
            }
            RoochMultiChainID::Ether => {
                let address = EthereumAddress::from_str(str)?;
                Ok(address.into())
            }
            RoochMultiChainID::Rooch => {
                let address = RoochAddress::from_str(str)?;
                Ok(address.into())
            }
            RoochMultiChainID::Nostr => {
                let pk = NostrPublicKey::from_str(str)?;
                Ok(pk.into())
            }
        }
    }

    pub fn is_rooch_address(&self) -> bool {
        self.multichain_id.is_rooch()
    }

    pub fn is_bitcoin_address(&self) -> bool {
        self.multichain_id.is_bitcoin()
    }

    pub fn to_original_string(&self) -> String {
        match self.multichain_id {
            RoochMultiChainID::Bitcoin => {
                let address = BitcoinAddress::try_from(self.clone()).unwrap();
                address.to_string()
            }
            RoochMultiChainID::Ether => {
                let address = EthereumAddress::try_from(self.clone()).unwrap();
                address.to_string()
            }
            RoochMultiChainID::Rooch => {
                let address = RoochAddress::try_from(self.clone()).unwrap();
                address.to_string()
            }
            RoochMultiChainID::Nostr => {
                let pk = NostrPublicKey::try_from(self.clone()).unwrap();
                pk.to_string()
            }
        }
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
            serializer.serialize_str(&self.to_string())
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
            let str = String::deserialize(deserializer)?;
            Self::from_str(&str).map_err(serde::de::Error::custom)
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

//Use multichain_id:original_address to represent multichain_id address,
//eth:0x1234.., btc:1px99y..., rooch:0x1234..
impl std::fmt::Display for MultiChainAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.multichain_id, self.to_original_string())
    }
}

impl FromStr for MultiChainAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(':').collect::<Vec<&str>>();
        if parts.len() != 2 {
            bail!("invalid multichain address {}", s);
        }
        let multichain_id = RoochMultiChainID::from_str(parts[0])?;
        Self::try_from_str_with_multichain_id(multichain_id, parts[1])
    }
}

impl MoveStructType for MultiChainAddress {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("multichain_address");
    const STRUCT_NAME: &'static IdentStr = ident_str!("MultiChainAddress");
}

impl MoveStructState for MultiChainAddress {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            RoochMultiChainID::type_layout(),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }
}

/// Rooch address type
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct RoochAddress(pub H256);

impl RoochAddress {
    /// RoochAddress length in bytes
    pub const LENGTH: usize = 32;

    /// RoochAddress length in bech32 string length: 5 hrp + 59 data
    pub const LENGTH_BECH32: usize = 64;

    /// RoochAddress length in hex string length: 0x + 64 data
    pub const LENGTH_HEX: usize = 66;

    pub fn is_vm_or_system_reserved_address(&self) -> bool {
        moveos_types::addresses::is_vm_or_system_reserved_address((*self).into())
    }

    pub fn to_bech32(&self) -> String {
        let data = self.0.as_bytes();
        bech32::encode::<Bech32m>(ROOCH_HRP, data).expect("bech32 encode should success")
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0.to_fixed_bytes()
    }

    pub fn from_bech32(bech32: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(bech32)?;
        anyhow::ensure!(hrp == ROOCH_HRP, "invalid rooch hrp");
        anyhow::ensure!(data.len() == Self::LENGTH, "invalid rooch address length");
        let hash = H256::from_slice(data.as_slice());
        Ok(Self(hash))
    }

    /// RoochAddress from_hex_literal support short hex string, such as 0x1, 0x2, 0x3
    pub fn from_hex_literal(literal: &str) -> Result<Self> {
        anyhow::ensure!(literal.starts_with("0x"), "Hex literal must start with 0x");

        let hex_len = literal.len() - 2;

        // If the string is too short, pad it
        if hex_len < Self::LENGTH * 2 {
            let mut hex_str = String::with_capacity(Self::LENGTH * 2);
            for _ in 0..Self::LENGTH * 2 - hex_len {
                hex_str.push('0');
            }
            hex_str.push_str(&literal[2..]);
            RoochAddress::from_hex(hex_str)
        } else {
            RoochAddress::from_hex(&literal[2..])
        }
    }

    /// RoochAddress to_hex_literal always return full hex string
    pub fn to_hex_literal(&self) -> String {
        format!("0x{:x}", self.0)
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| anyhow::anyhow!("Invalid address hex string"))
            .map(H256)
            .map(Self)
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", self.0)
    }
}

impl std::cmp::PartialEq<AccountAddress> for RoochAddress {
    fn eq(&self, other: &AccountAddress) -> bool {
        &self.0 .0 == other.deref()
    }
}

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

impl From<H256> for RoochAddress {
    fn from(hash: H256) -> Self {
        Self(hash)
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
        //Use bech32 as default display format
        write!(f, "{}", self.to_bech32())
    }
}

impl fmt::LowerHex for RoochAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in self.0.as_bytes() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl fmt::UpperHex for RoochAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in self.0.as_bytes() {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

impl fmt::Debug for RoochAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the address in bech32 and hex format for debug
        write!(f, "{}({})", self.to_bech32(), self.to_hex_literal())
    }
}

impl FromStr for RoochAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            RoochAddress::from_hex_literal(s)
        } else {
            RoochAddress::from_bech32(s)
        }
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
            //Use Move AccountAddress as default binary serialize format
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

impl From<BitcoinAddress> for RoochAddress {
    fn from(value: BitcoinAddress) -> Self {
        value.to_rooch_address()
    }
}

/// Ethereum address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd, Copy)]
#[serde_as]
pub struct EthereumAddress(pub H160);

impl fmt::Display for EthereumAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the Ethereum address as a hexadecimal string with a "0x" prefix
        write!(f, "{:#x}", self.0)
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

impl From<Secp256k1PublicKey> for EthereumAddress {
    fn from(value: Secp256k1PublicKey) -> Self {
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

/// The method used to distinguish bitcoin address payload type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum BitcoinAddressPayloadType {
    /// P2PKH address.
    PubkeyHash = 0,
    /// P2SH address.
    ScriptHash = 1,
    /// Segwit address.
    WitnessProgram = 2,
}

impl BitcoinAddressPayloadType {
    pub fn to_num(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for BitcoinAddressPayloadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

impl FromStr for BitcoinAddressPayloadType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let version = s
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("bitcoin address payload type parse error {} ", s))?;
        BitcoinAddressPayloadType::try_from(version)
    }
}

impl TryFrom<u8> for BitcoinAddressPayloadType {
    type Error = anyhow::Error;

    fn try_from(no: u8) -> std::result::Result<Self, Self::Error> {
        Ok(match no {
            0 => BitcoinAddressPayloadType::PubkeyHash,
            1 => BitcoinAddressPayloadType::ScriptHash,
            2 => BitcoinAddressPayloadType::WitnessProgram,
            _ => {
                return Err(anyhow::anyhow!(
                    "bitcoin address payload type is invalid {} ",
                    no
                ));
            }
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[serde_as]
pub struct BitcoinAddress {
    bytes: Vec<u8>,
}

impl Serialize for BitcoinAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            self.bytes.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for BitcoinAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            Self::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = Vec::<u8>::deserialize(deserializer)?;
            Ok(Self { bytes })
        }
    }
}

impl fmt::Debug for BitcoinAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for BitcoinAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the Bitcoin address as a hexadecimal string
        // Default format as bitcoin mainnet address
        let bitcoin_address = self
            .format(network::Network::Bitcoin.to_num())
            .map_err(|e| std::fmt::Error::custom(e.to_string()))?;
        write!(fmt, "{}", bitcoin_address)
    }
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

    pub fn get_pubkey_address_prefix(network: network::Network) -> u8 {
        if network::Network::Bitcoin == network {
            bitcoin::constants::PUBKEY_ADDRESS_PREFIX_MAIN
        } else {
            bitcoin::constants::PUBKEY_ADDRESS_PREFIX_TEST
        }
    }

    pub fn get_script_address_prefix(network: network::Network) -> u8 {
        if network::Network::Bitcoin == network {
            bitcoin::constants::SCRIPT_ADDRESS_PREFIX_MAIN
        } else {
            bitcoin::constants::SCRIPT_ADDRESS_PREFIX_TEST
        }
    }

    pub fn new_p2pkh(pubkey_hash: &bitcoin::PubkeyHash) -> Self {
        let mut bytes = [0; 21];
        bytes[0] = BitcoinAddressPayloadType::PubkeyHash.to_num();
        bytes[1..].copy_from_slice(&pubkey_hash[..]);
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn new_p2sh(script_hash: &bitcoin::ScriptHash) -> Self {
        let mut bytes = [0; 21];
        bytes[0] = BitcoinAddressPayloadType::ScriptHash.to_num();
        bytes[1..].copy_from_slice(&script_hash[..]);
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn new_witness_program(witness_program: &bitcoin::WitnessProgram) -> Self {
        // First byte is BitcoinAddress Payload type
        let mut bytes = vec![BitcoinAddressPayloadType::WitnessProgram.to_num()];
        // Second byte represents Version 0 or PUSHNUM_1-PUSHNUM_16
        bytes.push(witness_program.version().to_num());
        // Remain are Program data
        bytes.extend_from_slice(witness_program.program().as_bytes());
        Self { bytes }
    }

    /// The empty address is used to if we parse the address failed from the script
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Convert the Bitcoin address to Rooch address
    pub fn to_rooch_address(&self) -> RoochAddress {
        let mut hasher = Blake2b256::default();
        hasher.update(&self.bytes);
        let g_arr = hasher.finalize();
        RoochAddress(H256(g_arr.digest))
    }

    pub fn to_bitcoin_address<N: Into<network::Network>>(
        &self,
        network: N,
    ) -> Result<bitcoin::Address, anyhow::Error> {
        let network: network::Network = network.into();
        let network = bitcoin::network::Network::from(network);
        let payload_type = BitcoinAddressPayloadType::try_from(self.bytes[0])?;
        let addr = match payload_type {
            BitcoinAddressPayloadType::PubkeyHash => {
                let pubkey_hash = bitcoin::PubkeyHash::from_slice(&self.bytes[1..])?;
                bitcoin::address::Address::p2pkh(pubkey_hash, network)
            }
            BitcoinAddressPayloadType::ScriptHash => {
                let script_hash = bitcoin::ScriptHash::from_slice(&self.bytes[1..])?;
                bitcoin::address::Address::p2sh_from_hash(script_hash, network)
            }
            BitcoinAddressPayloadType::WitnessProgram => {
                let version = WitnessVersion::try_from(self.bytes[1])?;
                let witness_program = WitnessProgram::new(version, &self.bytes[2..])?;
                bitcoin::address::Address::from_witness_program(witness_program, network)
            }
        };
        Ok(addr)
    }

    pub fn pay_load_type(&self) -> BitcoinAddressPayloadType {
        BitcoinAddressPayloadType::try_from(self.bytes[0]).unwrap()
    }

    pub fn pay_load(&self) -> &[u8] {
        &self.bytes[1..]
    }

    pub fn script_pubkey(&self) -> Result<bitcoin::ScriptBuf> {
        let bitcoin_address = self.to_bitcoin_address(network::Network::Bitcoin)?;
        Ok(bitcoin_address.script_pubkey())
    }

    ///  Format the base58 as a hexadecimal string
    pub fn format<N: Into<network::Network>>(&self, network: N) -> Result<String, anyhow::Error> {
        if self.bytes.is_empty() {
            anyhow::bail!("bitcoin address is empty");
        }
        let network: network::Network = network.into();
        let payload_type = BitcoinAddressPayloadType::try_from(self.bytes[0])?;
        match payload_type {
            BitcoinAddressPayloadType::PubkeyHash => {
                let mut prefixed = [0; 21];
                prefixed[0] = Self::get_pubkey_address_prefix(network);
                prefixed[1..].copy_from_slice(&self.bytes[1..]);
                Ok(bs58::encode(&prefixed[..]).with_check().into_string())
            }
            BitcoinAddressPayloadType::ScriptHash => {
                let mut prefixed = [0; 21];
                prefixed[0] = Self::get_script_address_prefix(network);
                prefixed[1..].copy_from_slice(&self.bytes[1..]);
                Ok(bs58::encode(&prefixed[..]).with_check().into_string())
            }
            BitcoinAddressPayloadType::WitnessProgram => {
                let hrp = network.bech32_hrp();
                let version = WitnessVersion::try_from(self.bytes[1])?;

                let witness_program = WitnessProgram::new(version, &self.bytes[2..])?;
                let program: &[u8] = witness_program.program().as_ref();

                let mut address_formatter = String::new();
                encode_to_fmt_unchecked(&mut address_formatter, hrp, version.to_fe(), program)?;
                Ok(address_formatter)
            }
        }
    }

    pub fn is_witness(&self) -> bool {
        if self.bytes.is_empty() {
            return false;
        }
        let payload_type = BitcoinAddressPayloadType::try_from(self.bytes[0]).unwrap();
        payload_type == BitcoinAddressPayloadType::WitnessProgram
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
        let bitcoin_network = Network::from(network::Network::Regtest);

        let secp = Secp256k1::new();
        let p2pkh_address = Address::p2pkh(
            PrivateKey::generate(bitcoin_network).public_key(&secp),
            bitcoin_network,
        );
        let p2sh_address = Address::p2sh(
            Script::from_bytes(H160::random().as_bytes()),
            bitcoin_network,
        )
        .unwrap();

        let sk = PrivateKey::generate(bitcoin_network);
        let segwit_address = Address::p2wpkh(
            &CompressedPublicKey::from_private_key(&secp, &sk).unwrap(),
            bitcoin_network,
        );

        // Create an array of addresses bitcoin protocols
        let addresses = [p2pkh_address, p2sh_address, segwit_address];
        // Randomly select one of the addresses
        let mut rng = thread_rng();
        let selected_address = addresses.choose(&mut rng).unwrap().clone();
        // Return the randomly selected Bitcoin address
        selected_address.into()
    }
}

impl FromStr for BitcoinAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = Address::from_str(s)?.assume_checked();
        Ok(Self::from(address))
    }
}

impl From<bitcoin::Address> for BitcoinAddress {
    fn from(address: bitcoin::Address) -> Self {
        address.to_address_data().into()
    }
}

impl TryFrom<BitcoinAddress> for bitcoin::Address {
    type Error = anyhow::Error;

    fn try_from(value: BitcoinAddress) -> Result<Self, Self::Error> {
        value.to_bitcoin_address(network::Network::Bitcoin)
    }
}

impl From<&bitcoin::address::AddressData> for BitcoinAddress {
    fn from(payload: &bitcoin::address::AddressData) -> Self {
        match payload {
            bitcoin::address::AddressData::P2pkh { pubkey_hash } => Self::new_p2pkh(pubkey_hash),
            bitcoin::address::AddressData::P2sh { script_hash } => Self::new_p2sh(script_hash),
            bitcoin::address::AddressData::Segwit { witness_program } => {
                Self::new_witness_program(witness_program)
            }
            _ => BitcoinAddress::default(),
        }
    }
}

impl From<bitcoin::address::AddressData> for BitcoinAddress {
    fn from(payload: bitcoin::address::AddressData) -> Self {
        Self::from(&payload)
    }
}

impl From<bitcoin::ScriptBuf> for BitcoinAddress {
    fn from(script: bitcoin::ScriptBuf) -> Self {
        Self::from(&script)
    }
}

impl From<&bitcoin::ScriptBuf> for BitcoinAddress {
    fn from(script: &bitcoin::ScriptBuf) -> Self {
        let address_opt = bitcoin::address::Address::from_script(script, &Params::MAINNET).ok();
        let payload: Option<AddressData> = match address_opt {
            None => {
                if script.is_p2pk() {
                    let p2pk_pubkey = script.p2pk_public_key();
                    p2pk_pubkey.map(|pubkey| bitcoin::address::AddressData::P2pkh {
                        pubkey_hash: pubkey.pubkey_hash(),
                    })
                } else {
                    None
                }
            }
            Some(address) => Some(address.to_address_data()),
        };
        payload.map(|payload| payload.into()).unwrap_or_default()
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

// Ref: https://github.com/nostr-protocol/nips/blob/master/19.md
/// Nostr public key type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NostrPublicKey(XOnlyPublicKey);

impl NostrPublicKey {
    pub fn new(x_only_pk: XOnlyPublicKey) -> Self {
        Self(x_only_pk)
    }

    /// Convert from the Nostr XOnlyPublicKey to Bitcoin Taproot address. BIP-086.
    pub fn to_bitcoin_address<N: Into<network::Network>>(
        &self,
        network: N,
    ) -> Result<BitcoinAddress, anyhow::Error> {
        // get the network
        let network: network::Network = network.into();
        // change use of XOnlyPublicKey from nostr to bitcoin lib
        let internal_key = bitcoin::XOnlyPublicKey::from_slice(&self.0.serialize())?;
        // new verification crypto
        let secp = Secp256k1::verification_only();
        // new bitcoin taproot address
        let address = Address::p2tr(
            &secp,
            internal_key,
            None,
            bitcoin::network::Network::from(network),
        );
        // give it to rooch bitcoin struct
        Ok(BitcoinAddress::from(address))
    }
}

impl RoochSupportedAddress for NostrPublicKey {
    fn random() -> Self {
        Self(
            RoochKeyPair::generate_secp256k1()
                .public()
                .xonly_public_key()
                .unwrap(),
        )
    }
}

impl From<NostrPublicKey> for MultiChainAddress {
    fn from(pk: NostrPublicKey) -> Self {
        Self::new(RoochMultiChainID::Nostr, pk.0.serialize().to_vec())
    }
}

impl TryFrom<MultiChainAddress> for NostrPublicKey {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochMultiChainID::Nostr {
            return Err(anyhow::anyhow!(
                "multichain_id type {} is invalid",
                value.multichain_id
            ));
        }
        let pk = XOnlyPublicKey::from_slice(&value.raw_address)?;
        Ok(Self(pk))
    }
}

/// FromStr is FromBech32 here for NostrPublicKey
impl FromStr for NostrPublicKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pk = XOnlyPublicKey::from_bech32(s)?;
        Ok(Self(pk))
    }
}

impl fmt::Display for NostrPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_bech32().map_err(|_| fmt::Error)?)
    }
}

// Parsed Address, either a name or a numerical address, or Bitcoin Address
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ParsedAddress {
    Named(String),
    Numerical(RoochAddress),
    Bitcoin(BitcoinAddress),
}

impl Default for ParsedAddress {
    fn default() -> Self {
        Self::Named("default".to_string())
    }
}

impl ParsedAddress {
    pub fn into_rooch_address(
        self,
        mapping: &impl Fn(&str) -> Option<AccountAddress>,
    ) -> anyhow::Result<RoochAddress> {
        match self {
            Self::Named(n) => mapping(n.as_str())
                .map(Into::into)
                .ok_or_else(|| anyhow::anyhow!("Unbound named address: '{}'", n)),
            Self::Numerical(a) => Ok(a),
            Self::Bitcoin(a) => Ok(a.to_rooch_address()),
        }
    }

    pub fn into_account_address(
        self,
        mapping: &impl Fn(&str) -> Option<AccountAddress>,
    ) -> anyhow::Result<AccountAddress> {
        self.into_rooch_address(mapping).map(AccountAddress::from)
    }

    pub fn parse(s: &str) -> anyhow::Result<Self> {
        if s.starts_with("0x") {
            Ok(Self::Numerical(RoochAddress::from_hex_literal(s)?))
        } else if s.starts_with(ROOCH_HRP.as_str()) && s.len() == RoochAddress::LENGTH_BECH32 {
            Ok(Self::Numerical(RoochAddress::from_bech32(s)?))
        } else if s.starts_with(PREFIX_BECH32_PUBLIC_KEY) {
            Ok(Self::Bitcoin(NostrPublicKey::to_bitcoin_address(
                &NostrPublicKey::from_str(s)?,
                network::Network::Bitcoin.to_num(),
            )?))
        } else {
            match BitcoinAddress::from_str(s) {
                Ok(a) => Ok(Self::Bitcoin(a)),
                Err(_) => Ok(Self::Named(s.to_string())),
            }
        }
    }
}

impl FromStr for ParsedAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedAddress::parse(s)
    }
}

// TODO: Need a testcase to use the nostr address.
#[cfg(test)]
mod test {
    use super::*;
    use bitcoin::hex::DisplayHex;
    use bitcoin::ScriptBuf;
    use std::{fmt::Debug, vec};

    #[test]
    fn test_bech32() {
        let address = RoochAddress::random();
        let hex = address.to_hex_literal();
        let bech32 = address.to_bech32();
        println!("bech32: {}, hex: {}", bech32, hex);
        assert_eq!(bech32.len(), RoochAddress::LENGTH_BECH32);
        let address2 = RoochAddress::from_bech32(&bech32).unwrap();
        assert_eq!(address, address2);
        let address3 = RoochAddress::from_hex_literal(&hex).unwrap();
        assert_eq!(address, address3);
    }

    fn test_sdk_multi_chain_address(address_str: &str, expect_address_bytes: Vec<u8>) {
        let address = bitcoin::Address::from_str(address_str)
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();
        let bitcoin_address = BitcoinAddress::from(address);
        let multi_chain_address = MultiChainAddress::from(bitcoin_address);

        assert_eq!(expect_address_bytes, multi_chain_address.to_bytes())
    }

    #[test]
    fn test_sdk_address() {
        // native swgwit p2wpkh
        let p2wpkh_address_str = "bc1pq5ttgyqu5pmfn9aqt09d978mky2fndxr3ed3ntszta75g9q6xrlqlwyl0r";
        let p22pkh_expect_address_bytes: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 34, 2, 1, 5, 22, 180, 16, 28, 160, 118, 153, 151, 160, 91, 202,
            210, 248, 251, 177, 20, 153, 180, 195, 142, 91, 25, 174, 2, 95, 125, 68, 20, 26, 48,
            254,
        ];
        test_sdk_multi_chain_address(p2wpkh_address_str, p22pkh_expect_address_bytes);

        // nestd segwit p2sh-p2wpkh
        let p2sh_address_str = "39fVbRM2TNBdNZPYeAJM7sHCPKczaZL7LV";
        let p2sh_expect_address_bytes: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 21, 1, 87, 119, 69, 184, 41, 233, 101, 189, 166, 156, 217, 192,
            62, 9, 151, 234, 162, 97, 49, 192,
        ];
        test_sdk_multi_chain_address(p2sh_address_str, p2sh_expect_address_bytes);

        // taproot p2tr
        let p2tr_address_str = "bc1pq5ttgyqu5pmfn9aqt09d978mky2fndxr3ed3ntszta75g9q6xrlqlwyl0r";
        let p2tr_expect_address_bytes: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 34, 2, 1, 5, 22, 180, 16, 28, 160, 118, 153, 151, 160, 91, 202,
            210, 248, 251, 177, 20, 153, 180, 195, 142, 91, 25, 174, 2, 95, 125, 68, 20, 26, 48,
            254,
        ];
        test_sdk_multi_chain_address(p2tr_address_str, p2tr_expect_address_bytes);

        // legacy p2pkh
        let p2pkh_address_str = "15MJa2Jx2yA5iERwTKENY2WdWF3vnN6KVe";
        let p2pkh_expect_address_bytes: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 21, 0, 47, 183, 125, 16, 71, 244, 105, 179, 253, 132, 178, 184,
            60, 5, 68, 57, 97, 253, 162, 187,
        ];
        test_sdk_multi_chain_address(p2pkh_address_str, p2pkh_expect_address_bytes);
    }

    fn test_rooch_supported_address_roundtrip<T>()
    where
        T: RoochSupportedAddress
            + Clone
            + Debug
            + PartialEq
            + Eq
            + std::hash::Hash
            + std::fmt::Display
            + FromStr,
        <T as std::str::FromStr>::Err: Debug,
    {
        let address = T::random();
        let address_str = address.to_string();
        let address_from_str = T::from_str(&address_str).expect("parse address from str failed");
        assert_eq!(
            address, address_from_str,
            "address {} != {}",
            address, address_from_str
        );

        let multi_chain_address: MultiChainAddress = address.clone().into();
        let address2 = T::try_from(multi_chain_address.clone()).unwrap();
        assert_eq!(address, address2);
        let addr_str = multi_chain_address.to_string();
        let address3_result = MultiChainAddress::from_str(&addr_str);
        assert!(
            address3_result.is_ok(),
            "parse address {} failed, err:{:?}",
            addr_str,
            address3_result.err()
        );
        let address3 = address3_result.unwrap();
        assert_eq!(multi_chain_address, address3);
        let address4 = T::try_from(address3).unwrap();
        assert_eq!(address, address4);
    }

    #[test]
    fn test_address() {
        test_rooch_supported_address_roundtrip::<RoochAddress>();
        test_rooch_supported_address_roundtrip::<EthereumAddress>();
        test_rooch_supported_address_roundtrip::<BitcoinAddress>();
        // TODO: deal with hex and bech32 format
        // test_rooch_supported_address_roundtrip::<NostrPublicKey>();
    }

    fn test_rooch_address_roundtrip(rooch_address: RoochAddress) {
        let rooch_hex_str = rooch_address.to_hex_literal();
        //ensure the rooch to_hex_literal is hex with 0x prefix
        //and is full 32 bytes output
        assert!(rooch_hex_str.starts_with("0x"));
        assert_eq!(rooch_hex_str.len(), RoochAddress::LENGTH_HEX);
        let rooch_address_from_str = RoochAddress::from_str(&rooch_hex_str).unwrap();
        assert_eq!(rooch_address, rooch_address_from_str);

        let rooch_bech32_str = rooch_address.to_bech32();
        assert_eq!(rooch_bech32_str.len(), RoochAddress::LENGTH_BECH32);
        let rooch_address_from_bech32 = RoochAddress::from_bech32(&rooch_bech32_str).unwrap();
        assert_eq!(rooch_address, rooch_address_from_bech32);

        let json_str = serde_json::to_string(&rooch_address).unwrap();
        assert_eq!(format!("\"{}\"", rooch_bech32_str), json_str);
        let rooch_address_from_json: RoochAddress = serde_json::from_str(&json_str).unwrap();
        assert_eq!(rooch_address, rooch_address_from_json);

        let bytes = bcs::to_bytes(&rooch_address).unwrap();
        assert!(bytes.len() == 32);
        let rooch_address_from_bytes: RoochAddress = bcs::from_bytes(&bytes).unwrap();
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
            let multi_chain_address: MultiChainAddress = address.into();
            let multi_chain_address_serialized = serde_json::to_string(&multi_chain_address).unwrap();
            let multi_chain_address_deserialized: MultiChainAddress = serde_json::from_str(&multi_chain_address_serialized).unwrap();
            assert_eq!(multi_chain_address, multi_chain_address_deserialized);
        }

    }

    #[test]
    pub fn test_from_script() {
        let bytes = hex::decode("001497cdff4fd3ed6f885d54a52b79d7a2141072ae3f").unwrap();
        let script = Script::from_bytes(bytes.as_slice());
        // let address = Address::from_script(script, Network::Signet).unwrap();
        let address = Address::from_script(script, Network::Bitcoin).unwrap();
        //println!("{:?}", address.address_type());
        assert_eq!(
            address.address_type().unwrap(),
            bitcoin::AddressType::P2wpkh
        );
        println!("bitcoin address from script {}", address);
        assert_eq!(
            address.to_string(),
            // "tb1qjlxl7n7na4hcsh25554hn4azzsg89t3ljdldnj"
            "bc1qjlxl7n7na4hcsh25554hn4azzsg89t3lcty7gp"
        )
    }

    #[test]
    pub fn test_bitcoin_address() -> Result<()> {
        let bytes = hex::decode("020097cdff4fd3ed6f885d54a52b79d7a2141072ae3f").unwrap();
        let bitcoin_address = BitcoinAddress {
            bytes: bytes.clone(),
        };
        let address_str = bitcoin_address.format(network::Network::Bitcoin)?;
        println!("test_bitcoin_address bitcoin address {} ", address_str);
        let maddress = MultiChainAddress::new(RoochMultiChainID::Bitcoin, bytes.clone());

        let new_bitcoin_address = BitcoinAddress::from_str(address_str.as_str())?;
        let new_maddress = MultiChainAddress::try_from_str_with_multichain_id(
            RoochMultiChainID::Bitcoin,
            address_str.as_str(),
        )?;

        assert_eq!(maddress, new_maddress);
        assert_eq!(bitcoin_address, new_bitcoin_address);
        assert_eq!(address_str, "bc1qjlxl7n7na4hcsh25554hn4azzsg89t3lcty7gp");

        let btc_address = bitcoin::Address::from_str(&address_str).unwrap();
        let btc_address = btc_address.assume_checked();
        let btc_address2 = bitcoin_address.to_bitcoin_address(bitcoin::Network::Bitcoin)?;
        assert_eq!(btc_address, btc_address2);

        Ok(())
    }

    #[test]
    pub fn test_convert_bitcoin_address() -> Result<()> {
        // bitcoin regtest address
        let bytes =
            hex::decode("020145966003624094dae2deeb30815eedd38f96c45c3fdb1261f5d697fc4137e0de")
                .unwrap();
        let bitcoin_address = BitcoinAddress {
            bytes: bytes.clone(),
        };
        let address_str = bitcoin_address.format(network::Network::Bitcoin)?;
        let btc_address = bitcoin::Address::from_str(&address_str).unwrap();
        let btc_address = btc_address.assume_checked();
        let btc_address2 = bitcoin_address.to_bitcoin_address(network::Network::Bitcoin)?;
        assert_eq!(btc_address, btc_address2);
        println!(
            "test_convert_bitcoin_address bitcoin address {} ",
            address_str
        );
        println!(
            "test_convert_bitcoin_address raw address bytes {:?} ",
            bytes
        );
        println!(
            "test_convert_bitcoin_address raw address hex str {} ",
            bytes.to_lower_hex_string()
        );

        Ok(())
    }

    #[test]
    pub fn test_bitcoin_address_from_str() -> Result<()> {
        let bitcoin_address_str = "3MSqmLCmL5XW1PbUnabyLtkYdLXePGokCu";
        let bitcoin_address = BitcoinAddress::from_str(bitcoin_address_str)?;
        let bitcoin_address_format = bitcoin_address.format(network::Network::Bitcoin.to_num())?;
        println!(
            "test_bitcoin_address_from_str bitcoin address format {} ",
            bitcoin_address_format
        );

        assert_eq!(bitcoin_address_str, bitcoin_address_format);
        Ok(())
    }

    #[test]
    pub fn test_bitcoin_address_to_rooch_address() -> Result<()> {
        let bitcoin_address_strs = [
            "18cBEMRxXHqzWWCxZNtU91F5sbUNKhL5PX",
            "bc1q262qeyyhdakrje5qaux8m2a3r4z8sw8vu5mysh",
        ];
        let btc_addresses = bitcoin_address_strs
            .iter()
            .map(|s| BitcoinAddress::from_str(s).unwrap())
            .collect::<Vec<BitcoinAddress>>();

        let origin_btc_addresses = bitcoin_address_strs
            .iter()
            .map(|s| {
                bitcoin::Address::from_str(s)
                    .unwrap()
                    .require_network(bitcoin::Network::Bitcoin)
                    .unwrap()
            })
            .collect::<Vec<bitcoin::Address>>();

        for (btc_address, origin_btc_address) in
            btc_addresses.iter().zip(origin_btc_addresses.iter())
        {
            assert_eq!(btc_address.to_string(), origin_btc_address.to_string());
        }

        let rooch_addresses = btc_addresses
            .iter()
            .map(|btc_address| btc_address.to_rooch_address())
            .collect::<Vec<_>>();

        // for rooch_address in rooch_addresses.iter(){
        //     println!(
        //         "test_bitcoin_address_to_rooch_address rooch_address bech32:{} hex:{:#x}",
        //         rooch_address, rooch_address
        //     );
        // }

        assert_eq!(
            rooch_addresses
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>(),
            vec![
                "rooch1gxterelcypsyvh8cc9kg73dtnyct822ykx8pmu383qruzt4r93jshtc9fj".to_owned(),
                "rooch10lnft7hhq37vl0y97lwvkmzqt48fk76y0z88rfcu8zg6qm8qegfqx0rq2h".to_owned(),
            ]
        );

        assert_eq!(
            rooch_addresses
                .iter()
                .map(|a| a.to_hex_literal())
                .collect::<Vec<_>>(),
            vec![
                "0x419791e7f82060465cf8c16c8f45ab9930b3a944b18e1df2278807c12ea32c65".to_owned(),
                "0x7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12".to_owned(),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_from_bitcoin_tx_out() {
        // p2pk pubkey, not a script should return empty address
        let script =  ScriptBuf::from_hex("04f254e36949ec1a7f6e9548f16d4788fb321f429b2c7d2eb44480b2ed0195cbf0c3875c767fe8abb2df6827c21392ea5cc934240b9ac46c6a56d2bd13dd0b17a9").unwrap();
        let bitcoin_address = BitcoinAddress::from(script);
        assert_eq!(BitcoinAddress::default(), bitcoin_address);
        // p2pk script(outpoint: e1be133be54851d21f34666ae45211d6e76d60491cecfef17bba90731eb8f42a:0)
        let script =  ScriptBuf::from_hex("4104f254e36949ec1a7f6e9548f16d4788fb321f429b2c7d2eb44480b2ed0195cbf0c3875c767fe8abb2df6827c21392ea5cc934240b9ac46c6a56d2bd13dd0b17a9ac").unwrap();
        assert!(script.is_p2pk());
        let bitcoin_address = BitcoinAddress::from(script);
        assert_eq!(
            "1DR5CqnzFLDmPZ7h94RHTxLV7u19xkS5rn",
            bitcoin_address.to_string()
        );
        // p2pk script(outpoint: a3b0e9e7cddbbe78270fa4182a7675ff00b92872d8df7d14265a2b1e379a9d33:0)
        let script = ScriptBuf::from_hex("4104ea1feff861b51fe3f5f8a3b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324bad71f9904ac0ae7336507d785b17a2c115e427a32fac").unwrap();
        let bitcoin_address = BitcoinAddress::from(script);
        assert_eq!(
            "1BBz9Z15YpELQ4QP5sEKb1SwxkcmPb5TMs",
            bitcoin_address.to_string()
        );
        // p2ms script(outpoint: a353a7943a2b38318bf458b6af878b8384f48a6d10aad5b827d0550980abe3f0:0)
        let script = ScriptBuf::from_hex("0014f29f9316f0f1e48116958216a8babd353b491dae").unwrap();
        let bitcoin_address = BitcoinAddress::from(script);
        assert_eq!(
            "bc1q720ex9hs78jgz954sgt23w4ax5a5j8dwjj5kkm",
            bitcoin_address.to_string()
        );
        // invalid p2pk pubkey(outpoint: 41a3e9ee1910a2d40dd217bbc9fd3638c40d13c8fdda8a0aa9d49a2b4a199422:2)
        let script = ScriptBuf::from_hex(
            "036c6565662c206f6e7464656b2c2067656e6965742e2e2e202020202020202020",
        )
        .unwrap();
        let bitcoin_address = BitcoinAddress::from(script);
        assert_eq!(BitcoinAddress::default(), bitcoin_address);
        // invalid p2pk script(outpoint: 41a3e9ee1910a2d40dd217bbc9fd3638c40d13c8fdda8a0aa9d49a2b4a199422:2)
        let script = ScriptBuf::from_hex(
            "21036c6565662c206f6e7464656b2c2067656e6965742e2e2e202020202020202020ac",
        )
        .unwrap();
        assert!(script.is_p2pk());
        let bitcoin_address = BitcoinAddress::from(script);
        assert_eq!(BitcoinAddress::default(), bitcoin_address);
    }

    #[test]
    fn test_p2pkh() {
        let addr = BitcoinAddress::from_str("1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY").unwrap();
        assert_eq!(addr.pay_load_type(), BitcoinAddressPayloadType::PubkeyHash);
        let payload = addr.pay_load();
        println!("test_p2pkh payload len: {}", payload.len());
    }
}

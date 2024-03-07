// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::bitcoin::network;
use crate::{
    addresses::ROOCH_FRAMEWORK_ADDRESS,
    multichain_id::{MultiChainID, RoochMultiChainID},
};
use anyhow::{bail, Result};
use bitcoin::bech32::segwit::encode_to_fmt_unchecked;
use bitcoin::script::PushBytesBuf;
use bitcoin::{
    address::Address, secp256k1::Secp256k1, Network, PrivateKey, Script, WitnessProgram,
    WitnessVersion,
};

use ethers::types::H160;
use fastcrypto::secp256k1::Secp256k1PublicKey;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
#[cfg(any(test, feature = "fuzzing"))]
use moveos_types::h256;
use moveos_types::state::MoveState;
use moveos_types::{
    h256::H256,
    state::{MoveStructState, MoveStructType},
};
use nostr::secp256k1::XOnlyPublicKey;
use nostr::Keys;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
use rand::{seq::SliceRandom, thread_rng};
use serde::ser::Error;
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
                let address = NostrAddress::from_str(str)?;
                Ok(address.into())
            }
        }
    }

    pub fn is_rooch_address(&self) -> bool {
        self.multichain_id.is_rooch()
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
                let address = NostrAddress::try_from(self.clone()).unwrap();
                address.to_string()
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BitcoinAddress {
    bytes: Vec<u8>,
}

impl fmt::Display for BitcoinAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the Bitcoin address as a hexadecimal string
        // Default format as bitcoin mainnet address
        let bitcoin_address = self
            .format(network::Network::NetworkBitcoin.to_num())
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

    pub fn get_pubkey_address_prefix(network: u8) -> u8 {
        if network::Network::NetworkBitcoin.to_num() == network {
            bitcoin::constants::PUBKEY_ADDRESS_PREFIX_MAIN
        } else {
            bitcoin::constants::PUBKEY_ADDRESS_PREFIX_TEST
        }
    }

    pub fn get_script_address_prefix(network: u8) -> u8 {
        if network::Network::NetworkBitcoin.to_num() == network {
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
        // Third byte represents Version 0 or PUSHNUM_1-PUSHNUM_16
        bytes.push(witness_program.version().to_num());
        // Remain are Program data
        bytes.extend_from_slice(witness_program.program().as_bytes());
        Self { bytes }
    }

    /// The empty address is used to if we parse the address failed from the script
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    ///  Format the base58 as a hexadecimal string
    pub fn format(&self, network: u8) -> Result<String, anyhow::Error> {
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
                let hrp = network::Network::try_from(network)?.bech32_hrp();
                let version = WitnessVersion::try_from(self.bytes[1])?;
                let buf = PushBytesBuf::try_from(self.bytes[2..].to_vec())?;
                let witness_program = WitnessProgram::new(version, buf)?;
                let program: &[u8] = witness_program.program().as_ref();

                let mut address_formatter = String::new();
                encode_to_fmt_unchecked(&mut address_formatter, &hrp, version.to_fe(), program)?;
                Ok(address_formatter)
            }
        }
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
        let bitcoin_network = Network::from(network::Network::NetworkRegtest);

        let secp = Secp256k1::new();
        let p2pkh_address = Address::p2pkh(
            &PrivateKey::generate(bitcoin_network).public_key(&secp),
            bitcoin_network,
        );
        let p2sh_address = Address::p2sh(
            Script::from_bytes(H160::random().as_bytes()),
            bitcoin_network,
        )
        .unwrap();
        let segwit_address = Address::p2wpkh(
            &PrivateKey::generate(bitcoin_network).public_key(&secp),
            bitcoin_network,
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

impl FromStr for BitcoinAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = Address::from_str(s)?.assume_checked();
        Ok(Self::from(address))
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

impl FromStr for NostrAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = XOnlyPublicKey::from_str(s)?;
        Ok(Self(address))
    }
}

impl fmt::Display for NostrAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bitcoin::hex::DisplayHex;
    use std::fmt::Debug;

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
            let multi_chain_address: MultiChainAddress = address.clone().into();
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
        println!("bitcoin address from script {}", address.to_string());
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
        let address_str = bitcoin_address.format(network::Network::NetworkBitcoin.to_num())?;
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
        let address_str = bitcoin_address.format(network::Network::NetworkBitcoin.to_num())?;
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
}

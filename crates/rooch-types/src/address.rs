// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::str::FromStr;

use crate::coin::{Coin, ROOCH_COIN_ID};
use anyhow::Result;
use bech32::{FromBase32, ToBase32};
use bitcoin::{
    hashes::{hash160, Hash},
    PubkeyHash,
};
use ethers::types::H160;
use move_core_types::account_address::AccountAddress;
use moveos_types::h256::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The address type that Rooch supports
pub trait RoochSupportedAddress:
    Into<MultiChainAddress> + TryFrom<MultiChainAddress, Error = anyhow::Error>
{
    fn random() -> Self;
}

/// Multi chain address representation
/// The address is distinguished by the coin id, coin id standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiChainAddress {
    pub coin: Coin,
    pub address: Vec<u8>,
}

impl MultiChainAddress {
    pub(crate) fn new(coin_id: u32, address: Vec<u8>) -> Result<Self> {
        let coin = Coin::try_from(coin_id)
            .map_err(|e| anyhow::anyhow!("coin id {} is invalid, {}", coin_id, e))?;
        Ok(Self { coin, address })
    }

    pub fn from_bech32(bech32: &str) -> Result<Self> {
        let (hrp, data, variant) = bech32::decode(bech32)?;
        if variant != bech32::Variant::Bech32 {
            return Err(anyhow::anyhow!("invalid bech32 variant"));
        }
        let version = data.first().map(|u| u.to_u8());
        anyhow::ensure!(version.filter(|v| *v == 1u8).is_some(), "expect version 1");

        let coin = Coin::try_from(hrp.as_str())
            .map_err(|e| anyhow::anyhow!("coin id {} is invalid, {}", hrp, e))?;
        let address = Vec::<u8>::from_base32(&data[1..])?;
        Ok(Self { coin, address })
    }

    pub fn to_bech32(&self) -> String {
        let mut data = self.address.to_base32();
        //A Bech32 string consists of a human-readable part (HRP), a separator (the character '1'), and a data part
        data.insert(
            0,
            bech32::u5::try_from_u8(1).expect("1 to u8 should success"),
        );
        bech32::encode(
            &self.coin.symbol.to_lowercase(),
            data,
            bech32::Variant::Bech32,
        )
        .expect("bech32 encode should success")
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
            struct Value(u32, Vec<u8>);
            let value = Value(self.coin.id, self.address.clone());
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
            struct Value(u32, Vec<u8>);
            let value = Value::deserialize(deserializer)?;
            Self::new(value.0, value.1).map_err(serde::de::Error::custom)
        }
    }
}

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
        Self::new(ROOCH_COIN_ID, address.0.as_bytes().to_vec())
            .expect("RoochAddress to MultiChainAddress should success")
    }
}

impl TryFrom<MultiChainAddress> for RoochAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.coin.id != ROOCH_COIN_ID {
            return Err(anyhow::anyhow!("coin id {} is invalid", value.coin.id));
        }
        Ok(Self(H256::from_slice(&value.address)))
    }
}

// ==== Display and FromStr, Deserialize and Serialize ====
// Should keep consistent with AccountAddress

impl fmt::Display for RoochAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}", AccountAddress::from(*self))
    }
}

impl fmt::Debug for RoochAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", AccountAddress::from(*self))
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

/// Ethereum address type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EthereumAddress(pub H160);

impl RoochSupportedAddress for EthereumAddress {
    fn random() -> Self {
        Self(H160::random())
    }
}

impl From<EthereumAddress> for MultiChainAddress {
    fn from(address: EthereumAddress) -> Self {
        Self::new(slip44::Coin::Ether.id(), address.0.as_bytes().to_vec())
            .expect("EthereumAddress to MultiChainAddress should success")
    }
}

impl TryFrom<MultiChainAddress> for EthereumAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.coin.id != slip44::Coin::Ether.id() {
            return Err(anyhow::anyhow!("coin id {} is invalid", value.coin.id));
        }
        Ok(Self(H160::from_slice(&value.address)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitcoinAddress(pub bitcoin::Address);

impl RoochSupportedAddress for BitcoinAddress {
    fn random() -> Self {
        Self(bitcoin::Address::new(
            bitcoin::Network::Bitcoin,
            bitcoin::address::Payload::PubkeyHash(PubkeyHash::from_raw_hash(
                hash160::Hash::from_slice(H160::random().as_bytes()).unwrap(),
            )),
        ))
    }
}

impl From<BitcoinAddress> for MultiChainAddress {
    fn from(address: BitcoinAddress) -> Self {
        Self::new(
            slip44::Coin::Bitcoin.id(),
            address.0.to_string().into_bytes(),
        )
        .expect("BitcoinAddress to MultiChainAddress should success")
    }
}

impl TryFrom<MultiChainAddress> for BitcoinAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.coin.id != slip44::Coin::Bitcoin.id() {
            return Err(anyhow::anyhow!("coin id {} is invalid", value.coin.id));
        }
        let addr = bitcoin::Address::from_str(&String::from_utf8(value.address)?)
            .map_err(|e| anyhow::anyhow!("invalid bitcoin address, {}", e))?;

        Ok(Self(addr.require_network(bitcoin::Network::Bitcoin)?))
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
    }
}

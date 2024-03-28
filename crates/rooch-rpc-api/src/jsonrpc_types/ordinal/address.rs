use std::{
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
};

use bitcoin::{
    base58,
    bech32::{self, hrp, Hrp},
    constants::{
        PUBKEY_ADDRESS_PREFIX_MAIN, PUBKEY_ADDRESS_PREFIX_TEST, SCRIPT_ADDRESS_PREFIX_MAIN,
        SCRIPT_ADDRESS_PREFIX_TEST,
    },
    hex::write_err,
    script::PushBytesBuf,
    PubkeyHash, ScriptHash, WitnessProgram, WitnessVersion,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::jsonrpc_types::StrView;

use super::network::{
    NetworkCheckedView, NetworkUncheckedView, NetworkValidationView, NetworkView,
};

pub type PubkeyHashView = StrView<PubkeyHash>;
pub type ScriptHashView = StrView<ScriptHash>;
pub type WitnessProgramView = StrView<WitnessProgram>;

impl FromStr for PubkeyHashView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(PubkeyHash::from_str(s)?))
    }
}

impl From<PubkeyHashView> for PubkeyHash {
    fn from(value: PubkeyHashView) -> Self {
        value.0
    }
}

impl std::fmt::Display for PubkeyHashView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl FromStr for ScriptHashView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(ScriptHash::from_str(s)?))
    }
}

impl From<ScriptHashView> for ScriptHash {
    fn from(value: ScriptHashView) -> Self {
        value.0
    }
}

impl std::fmt::Display for ScriptHashView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl FromStr for WitnessProgramView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pb = PushBytesBuf::new();
        pb.extend_from_slice(s.as_bytes())?;
        Ok(StrView(WitnessProgram::new(WitnessVersion::V0, pb)?))
    }
}

impl From<WitnessProgramView> for WitnessProgram {
    fn from(value: WitnessProgramView) -> Self {
        value.0
    }
}

impl std::fmt::Display for WitnessProgramView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// The method used to produce an address.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[non_exhaustive]
pub enum PayloadView {
    /// P2PKH address.
    PubkeyHash(PubkeyHashView),
    /// P2SH address.
    ScriptHash(ScriptHashView),
    /// Segwit address.
    WitnessProgram(WitnessProgramView),
}

/// The inner representation of an address, without the network validation tag.
///
/// An `Address` is composed of a payload and a network. This struct represents the inner
/// representation of an address without the network validation tag, which is used to ensure that
/// addresses are used only on the appropriate network.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
struct AddressInnerView {
    payload: PayloadView,
    network: NetworkView,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, JsonSchema)]
///
/// The `#[repr(transparent)]` attribute is used to guarantee that the layout of the
/// `Address` struct is the same as the layout of the `AddressInner` struct. This attribute is
/// an implementation detail and users should not rely on it in their code.
///
#[repr(transparent)]
pub struct AddressView<V = NetworkCheckedView>(AddressInnerView, PhantomData<V>)
where
    V: NetworkValidationView;

struct DisplayUnchecked<'a, N: NetworkValidationView>(&'a AddressView<N>);

impl<N: NetworkValidationView> fmt::Display for DisplayUnchecked<'_, N> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_internal(fmt)
    }
}

impl<N: NetworkValidationView> serde::Serialize for AddressView<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&DisplayUnchecked(self))
    }
}

/// The different types of addresses.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[non_exhaustive]
pub enum AddressTypeView {
    /// Pay to pubkey hash.
    P2pkh,
    /// Pay to script hash.
    P2sh,
    /// Pay to witness pubkey hash.
    P2wpkh,
    /// Pay to witness script hash.
    P2wsh,
    /// Pay to taproot.
    P2tr,
}

impl fmt::Display for AddressTypeView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            AddressTypeView::P2pkh => "p2pkh",
            AddressTypeView::P2sh => "p2sh",
            AddressTypeView::P2wpkh => "p2wpkh",
            AddressTypeView::P2wsh => "p2wsh",
            AddressTypeView::P2tr => "p2tr",
        })
    }
}

/// Address type is either invalid or not supported in rust-bitcoin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct UnknownAddressTypeErrorView(pub String);

impl fmt::Display for UnknownAddressTypeErrorView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_err!(f, "failed to parse {} as address type", self.0; self)
    }
}

impl std::error::Error for UnknownAddressTypeErrorView {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl FromStr for AddressTypeView {
    type Err = UnknownAddressTypeErrorView;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p2pkh" => Ok(AddressTypeView::P2pkh),
            "p2sh" => Ok(AddressTypeView::P2sh),
            "p2wpkh" => Ok(AddressTypeView::P2wpkh),
            "p2wsh" => Ok(AddressTypeView::P2wsh),
            "p2tr" => Ok(AddressTypeView::P2tr),
            _ => Err(UnknownAddressTypeErrorView(s.to_string().to_owned())),
        }
    }
}

/// A utility struct to encode an address payload with the given parameters.
/// This is a low-level utility struct. Consider using `Address` instead.
pub struct AddressEncodingView<'a> {
    /// The address payload to encode.
    pub payload: &'a PayloadView,
    /// base58 version byte for p2pkh payloads (e.g. 0x00 for "1..." addresses).
    pub p2pkh_prefix: u8,
    /// base58 version byte for p2sh payloads (e.g. 0x05 for "3..." addresses).
    pub p2sh_prefix: u8,
    /// The bech32 human-readable part.
    pub hrp: Hrp,
}

/// Formats bech32 as upper case if alternate formatting is chosen (`{:#}`).
impl<'a> fmt::Display for AddressEncodingView<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.payload {
            PayloadView::PubkeyHash(hash) => {
                let mut prefixed = [0; 21];
                prefixed[0] = self.p2pkh_prefix;
                prefixed[1..].copy_from_slice(&hash.0[..]);
                base58::encode_check_to_fmt(fmt, &prefixed[..])
            }
            PayloadView::ScriptHash(hash) => {
                let mut prefixed = [0; 21];
                prefixed[0] = self.p2sh_prefix;
                prefixed[1..].copy_from_slice(&hash.0[..]);
                base58::encode_check_to_fmt(fmt, &prefixed[..])
            }
            PayloadView::WitnessProgram(witness_program) => {
                let hrp = &self.hrp;
                let version = witness_program.0.version().to_fe();
                let program = witness_program.0.program().as_bytes();

                if fmt.alternate() {
                    bech32::segwit::encode_upper_to_fmt_unchecked(fmt, hrp, version, program)
                } else {
                    bech32::segwit::encode_lower_to_fmt_unchecked(fmt, hrp, version, program)
                }
            }
        }
    }
}

/// Methods on [`Address`] that can be called on both `Address<NetworkChecked>` and
/// `Address<NetworkUnchecked>`.
impl<V: NetworkValidationView> AddressView<V> {
    /// Returns a reference to the payload of this address.
    pub fn payload(&self) -> &PayloadView {
        &self.0.payload
    }

    /// Returns a reference to the network of this address.
    pub fn network(&self) -> &NetworkView {
        &self.0.network
    }

    /// Returns a reference to the unchecked address, which is dangerous to use if the address
    /// is invalid in the context of `NetworkUnchecked`.
    pub fn as_unchecked(&self) -> &AddressView<NetworkUncheckedView> {
        unsafe { &*(self as *const AddressView<V> as *const AddressView<NetworkUncheckedView>) }
    }

    /// Extracts and returns the network and payload components of the `Address`.
    pub fn into_parts(self) -> (NetworkView, PayloadView) {
        let AddressInnerView { payload, network } = self.0;
        (network, payload)
    }

    /// Format the address for the usage by `Debug` and `Display` implementations.
    fn fmt_internal(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let p2pkh_prefix = match self.network() {
            NetworkView::Bitcoin => PUBKEY_ADDRESS_PREFIX_MAIN,
            NetworkView::Testnet | NetworkView::Signet | NetworkView::Regtest => {
                PUBKEY_ADDRESS_PREFIX_TEST
            }
        };
        let p2sh_prefix = match self.network() {
            NetworkView::Bitcoin => SCRIPT_ADDRESS_PREFIX_MAIN,
            NetworkView::Testnet | NetworkView::Signet | NetworkView::Regtest => {
                SCRIPT_ADDRESS_PREFIX_TEST
            }
        };
        let hrp = match self.network() {
            NetworkView::Bitcoin => hrp::BC,
            NetworkView::Testnet | NetworkView::Signet => hrp::TB,
            NetworkView::Regtest => hrp::BCRT,
        };
        let encoding = AddressEncodingView {
            payload: self.payload(),
            p2pkh_prefix,
            p2sh_prefix,
            hrp,
        };

        encoding.fmt(fmt)
    }

    /// Create new address from given components, infering the network validation
    /// marker type of the address.
    #[inline]
    pub fn new(network: NetworkView, payload: PayloadView) -> Self {
        Self(AddressInnerView { network, payload }, PhantomData)
    }
}

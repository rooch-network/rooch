// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use bech32::Hrp;
use bitcoin::XOnlyPublicKey;

pub const PREFIX_BECH32_PUBLIC_KEY: &str = "npub";
pub const NPUB: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_PUBLIC_KEY);

pub trait ToBech32 {
    type Err;
    fn to_bech32(&self) -> Result<String, Self::Err>;
}

impl ToBech32 for XOnlyPublicKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.serialize();
        Ok(bech32::encode::<bech32::Bech32>(NPUB, &data)?)
    }
}

pub trait FromBech32: Sized {
    type Err;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: Into<String>;
}

impl FromBech32 for XOnlyPublicKey {
    type Err = Error;

    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let (hrp, data) = bech32::decode(&s.into())?;
        if hrp != NPUB {
            return Err(Error::msg("Invalid HRP"));
        }
        Ok(XOnlyPublicKey::from_slice(&data)?)
    }
}

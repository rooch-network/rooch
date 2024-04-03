// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Code from https://github.com/ordinals/ord/

use crate::natives::ord::height::Height;
use bitcoin::constants::SUBSIDY_HALVING_INTERVAL;
use serde::{Deserialize, Serialize};

/// How many satoshis are in "one bitcoin".
pub const COIN_VALUE: u64 = 100_000_000;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, PartialOrd)]
pub(crate) struct Epoch(pub(crate) u32);

impl Epoch {
    pub(crate) const FIRST_POST_SUBSIDY: Epoch = Self(33);

    pub(crate) fn subsidy(self) -> u64 {
        if self < Self::FIRST_POST_SUBSIDY {
            (50 * COIN_VALUE) >> self.0
        } else {
            0
        }
    }

    pub(crate) fn starting_height(self) -> Height {
        Height(self.0 * SUBSIDY_HALVING_INTERVAL)
    }
}

impl PartialEq<u32> for Epoch {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl From<Height> for Epoch {
    fn from(height: Height) -> Self {
        Self(height.0 / SUBSIDY_HALVING_INTERVAL)
    }
}

#[cfg(test)]
mod tests {
    use crate::natives::ord::epoch::Epoch;
    use crate::natives::ord::height::Height;
    use bitcoin::constants::SUBSIDY_HALVING_INTERVAL;

    #[test]
    fn subsidy() {
        assert_eq!(Epoch(0).subsidy(), 5000000000);
        assert_eq!(Epoch(1).subsidy(), 2500000000);
        assert_eq!(Epoch(32).subsidy(), 1);
        assert_eq!(Epoch(33).subsidy(), 0);
    }

    #[test]
    fn starting_height() {
        assert_eq!(Epoch(0).starting_height(), 0);
        assert_eq!(Epoch(1).starting_height(), SUBSIDY_HALVING_INTERVAL);
        assert_eq!(Epoch(2).starting_height(), SUBSIDY_HALVING_INTERVAL * 2);
    }

    #[test]
    fn from_height() {
        assert_eq!(Epoch::from(Height(0)), 0);
        assert_eq!(Epoch::from(Height(SUBSIDY_HALVING_INTERVAL)), 1);
        assert_eq!(Epoch::from(Height(SUBSIDY_HALVING_INTERVAL) + 1), 1);
    }

    #[test]
    fn eq() {
        assert_eq!(Epoch(0), 0);
        assert_eq!(Epoch(100), 100);
    }

    #[test]
    fn first_post_subsidy() {
        assert_eq!(Epoch::FIRST_POST_SUBSIDY.subsidy(), 0);
        assert!((Epoch(Epoch::FIRST_POST_SUBSIDY.0 - 1)).subsidy() > 0);
    }
}

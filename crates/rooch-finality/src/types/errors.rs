// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("block not found")]
    BlockNotFound,

    #[error("no FP has voting power for the consumer chain")]
    NoFpHasVotingPower,

    #[error("BTC staking is not activated for the consumer chain")]
    BtcStakingNotActivated,

    #[error("BTC staking activated timestamp not found")]
    ActivatedTimestampNotFound,
}

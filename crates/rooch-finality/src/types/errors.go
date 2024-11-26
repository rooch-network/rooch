package types

import "errors"

var (
	ErrBlockNotFound              = errors.New("block not found")
	ErrNoFpHasVotingPower         = errors.New("no FP has voting power for the consumer chain")
	ErrBtcStakingNotActivated     = errors.New("BTC staking is not activated for the consumer chain")
	ErrActivatedTimestampNotFound = errors.New("BTC staking activated timestamp not found")
)

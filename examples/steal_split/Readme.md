# Steal Or Split Example

This is an example about stealing or splitting, and you can try using this example to learn how to build a simple game
in Rooch.

## Getting started

This example implements a game where the creator can initiate a game, and Player 1 and Player 2 can input their choices.
Within a specified time, if both players submit their choices and reveal the answers, rewards will be distributed based
on their selections. If neither player submits and reveals their answers within the specified time, the rewards will be
returned to the creator.

## Rule

- split - split: they will each receive half of the prize pool as a reward.
- split - steal: The player who chooses to steal will receive all the rewards in the prize pool.
- steal - steal: Neither of the two players will receive any rewards, and the rewards will be sent to the creator.
- If a player does not reveal their answer, the reward will be given to the player who revealed their answer.
- If there is no winner before the timeout, the reward will be given to the creator.

## Run Test

Due to the absence of the coin and timestamp modules in the current network, it is advisable to run this example as a
local test.

```shell
rooch move test 
```

Get:

```shell
INCLUDING DEPENDENCY MoveStdlib
INCLUDING DEPENDENCY MoveosStdlib
INCLUDING DEPENDENCY RoochFramework
BUILDING examples/steal_split
Running Move unit tests
[ PASS    ] 0x42::rooch_examples::test_create_game
[ PASS    ] 0x42::rooch_examples::test_get_next_game_id
[ PASS    ] 0x42::rooch_examples::test_init
[ PASS    ] 0x42::rooch_examples::test_init_again
[ PASS    ] 0x42::rooch_examples::test_make_decision
[ PASS    ] 0x42::rooch_examples::test_make_decision_decision_hash_is_none
[ PASS    ] 0x42::rooch_examples::test_make_decision_incorrect_hash_value
[ PASS    ] 0x42::rooch_examples::test_make_decision_salt_hash_is_none
[ PASS    ] 0x42::rooch_examples::test_release_funds_after_expiration_transfer_to_creator
[ PASS    ] 0x42::rooch_examples::test_release_funds_after_expiration_transfer_to_player_one
[ PASS    ] 0x42::rooch_examples::test_reveal_decision_both_players_steal
[ PASS    ] 0x42::rooch_examples::test_reveal_decision_player_one_does_not_have_a_decision_submitted
[ PASS    ] 0x42::rooch_examples::test_reveal_decision_player_one_steals
[ PASS    ] 0x42::rooch_examples::test_reveal_decision_player_two_steals
[ PASS    ] 0x42::rooch_examples::test_reveal_decision_split
[ PASS    ] 0x42::rooch_examples::test_submit_decision
[ PASS    ] 0x42::rooch_examples::test_submit_decision_player_one_has_a_decision_submitted
Test result: OK. Total tests: 17; passed: 17; failed: 0
Success
```


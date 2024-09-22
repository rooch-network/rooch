// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::bitseed::generator::{Generator, InscribeGenerateOutput, InscribeSeed};
use bitcoin::Address;
use move_core_types::u256::U256;

pub struct RandomAmountGenerator;

impl Generator for RandomAmountGenerator {
    fn inscribe_generate(
        &self,
        _deploy_args: &[u8],
        seed: &InscribeSeed,
        _recipient: &Address,
        _user_input: Option<String>,
    ) -> InscribeGenerateOutput {
        let hash = seed.seed();
        let min = U256::from(1u64);
        let max = U256::from(100u64);
        let amount = (U256::from_le_bytes(&hash.0) % (max - min) + min).unchecked_as_u64();
        InscribeGenerateOutput {
            amount,
            attributes: None,
            content: None,
        }
    }

    fn inscribe_verify(
        &self,
        deploy_args: &[u8],
        seed: &InscribeSeed,
        recipient: &Address,
        user_input: Option<String>,
        inscribe_output: InscribeGenerateOutput,
    ) -> bool {
        let output = self.inscribe_generate(deploy_args, seed, recipient, user_input);
        output == inscribe_output
    }
}

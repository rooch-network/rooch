// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{path::Path, str::FromStr};

use anyhow::Result;
use bitcoin::Txid;

// Load the babylon staking transactions exported file

// https://github.com/babylonlabs-io/staking-indexer

// Transaction Hash,Staking Output Index,Inclusion Height,Staker Public Key,Staking Time,Finality Provider Public Key,Is Overflow,Staking Value
// 8440304144a4585d80b60888ba58944f3c626d5c2a813b8955052b2daac20b00,0,864791,04bd117663e6970dad57769a9105bf72f8f7ec162b8e44bf597f41babe5cf8a3,64000,fc8a5b9930c3383e94bd940890e93cfcf95b2571ad50df8063b7011f120b918a,true,4800000
// ffaae2983630d3d51fac15180e2f89c1ae237e3648e11c5ec506113e78216e00,0,864791,3f1713f12f5ce2269c3360454fd552c77994f287f006b8f7e4c215b5f57a47ed,64000,db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923,true,1345800
// a1fa47d149457a994d2199ceffc43793eb18287864a6b7314c14ba3649f07000,0,864791,ef548602c263dc77b3c75ebb82edae9f1f57c16b6551c40179e9eb942b454be6,64000,742f1eb3c7fdbd327fa44fcdddf17645d9c6b1287ea97463e046508234fa7537,true,600000
// 7487946cb0598179b805ce73575bb22f99b2ca49d213bf57047ea864dc2f7800,0,864791,c749e4aa8436dc738373f1ccc9570ce9fe8a1d70bae1c25dac71f8e6e0c699ed,64000,0f5c19935a08f661a1c4dfeb5e51ce7f0cfcf4d2eeb405fe4c7d7bd668fc85e4,true,564500

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BBNStakingTxRecord {
    pub transaction_hash: String,
    pub staking_output_index: u32,
    pub inclusion_height: u64,
    pub staker_public_key: String,
    pub staking_time: u16,
    pub finality_provider_public_key: String,
    pub is_overflow: bool,
    pub staking_value: u64,
}
impl BBNStakingTxRecord {
    pub fn load_bbn_staking_txs<P: AsRef<Path>>(
        file_path: P,
        block_height: u64,
    ) -> Result<Vec<BBNStakingTxRecord>> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_path.as_ref())?;

        let mut txs = vec![];
        for result in rdr.records() {
            let record = result?;
            let tx: BBNStakingTxRecord = record.deserialize(None)?;
            if tx.inclusion_height == block_height {
                txs.push(tx);
            }
        }
        Ok(txs)
    }

    pub fn txid(&self) -> Txid {
        Txid::from_str(&self.transaction_hash).unwrap()
    }

    pub fn staker_public_key(&self) -> Vec<u8> {
        hex::decode(&self.staker_public_key).unwrap()
    }

    pub fn finality_provider_public_key(&self) -> Vec<u8> {
        hex::decode(&self.finality_provider_public_key).unwrap()
    }
}

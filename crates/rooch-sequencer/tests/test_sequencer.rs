// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::{system::ActorSystem, IntoActor};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_genesis::RoochGenesis;
use rooch_sequencer::{actor::sequencer::SequencerActor, proxy::SequencerProxy};
use rooch_types::{
    crypto::RoochKeyPair,
    transaction::{LedgerTxData, RoochTransaction},
};

fn init_rooch_db(opt: &RoochOpt) -> Result<RoochDB> {
    let rooch_db = RoochDB::init(opt.store_config())?;
    let network = opt.network();
    let genesis = RoochGenesis::build(network)?;
    genesis.init_genesis(&rooch_db)?;
    Ok(rooch_db)
}

#[test]
fn test_sequencer() -> Result<()> {
    let opt = RoochOpt::new_with_temp_store()?;
    let mut last_tx_order = 0;
    {
        let rooch_db = init_rooch_db(&opt)?;
        let sequencer_key = RoochKeyPair::generate_secp256k1();
        let mut sequencer = SequencerActor::new(sequencer_key, rooch_db.rooch_store)?;
        assert_eq!(sequencer.last_order(), last_tx_order);
        for _ in 0..10 {
            let tx_data = LedgerTxData::L2Tx(RoochTransaction::mock());
            let ledger_tx = sequencer.sequence(tx_data)?;
            assert_eq!(ledger_tx.sequence_info.tx_order, last_tx_order + 1);
            last_tx_order = ledger_tx.sequence_info.tx_order;
        }
        assert_eq!(sequencer.last_order(), last_tx_order);
    }
    // load from db again
    {
        let rooch_db = RoochDB::init(opt.store_config())?;
        let sequencer_key = RoochKeyPair::generate_secp256k1();
        let mut sequencer = SequencerActor::new(sequencer_key, rooch_db.rooch_store)?;
        assert_eq!(sequencer.last_order(), last_tx_order);
        let tx_data = LedgerTxData::L2Tx(RoochTransaction::mock());
        let ledger_tx = sequencer.sequence(tx_data)?;
        assert_eq!(ledger_tx.sequence_info.tx_order, last_tx_order + 1);
    }
    Ok(())
}

// test concurrent
// Build a sequencer actor and sequence transactions concurrently
#[tokio::test]
async fn test_sequencer_concurrent() -> Result<()> {
    let opt = RoochOpt::new_with_temp_store()?;
    let rooch_db = init_rooch_db(&opt)?;
    let sequencer_key = RoochKeyPair::generate_secp256k1();

    let actor_system = ActorSystem::global_system();

    let sequencer = SequencerActor::new(sequencer_key, rooch_db.rooch_store)?
        .into_actor(Some("Sequencer"), &actor_system)
        .await?;
    let sequencer_proxy = SequencerProxy::new(sequencer.into());

    // start n thread to sequence
    let n = 10;
    let mut handles = vec![];
    for _ in 0..n {
        let sequencer_proxy = sequencer_proxy.clone();
        //Use tokio to spawn a new async task
        let handle = tokio::task::spawn(async move {
            for _ in 0..n {
                let tx_data = LedgerTxData::L2Tx(RoochTransaction::mock());
                let _ = sequencer_proxy.sequence_transaction(tx_data).await.unwrap();
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await?;
    }

    let sequencer_order = sequencer_proxy.get_sequencer_order().await?;
    assert_eq!(sequencer_order, n * n);

    Ok(())
}

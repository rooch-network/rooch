// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::{system::ActorSystem, IntoActor};
use metrics::RegistryService;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::{StoreInstance, CF_METRICS_REPORT_PERIOD_MILLIS};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_genesis::RoochGenesis;
use rooch_sequencer::{actor::sequencer::SequencerActor, proxy::SequencerProxy};
use rooch_types::{
    crypto::RoochKeyPair,
    service_status::ServiceStatus,
    transaction::{LedgerTxData, RoochTransaction},
};
use std::time::Duration;

fn init_rooch_db(opt: &RoochOpt, registry: &Registry) -> Result<RoochDB> {
    DBMetrics::init(registry);
    let store_instance = RoochDB::generate_store_instance(opt.store_config(), registry)?;
    init_rooch_db_with_instance(opt, store_instance, registry)
}

fn init_rooch_db_with_instance(
    opt: &RoochOpt,
    instance: StoreInstance,
    registry: &Registry,
) -> Result<RoochDB> {
    let rooch_db = RoochDB::init_with_instance(opt.store_config(), instance, registry)?;
    let network = opt.network();
    let _genesis = RoochGenesis::load_or_init(network, &rooch_db)?;
    Ok(rooch_db)
}

#[tokio::test]
async fn test_sequencer() -> Result<()> {
    let opt = RoochOpt::new_with_temp_store()?;
    let mut last_tx_order = 0;
    let registry_service = RegistryService::default();
    {
        let mut store_instance = RoochDB::generate_store_instance(
            opt.store_config(),
            &registry_service.default_registry(),
        )?;
        let rooch_db = init_rooch_db_with_instance(
            &opt,
            store_instance.clone(),
            &registry_service.default_registry(),
        )?;
        let sequencer_key = RoochKeyPair::generate_secp256k1();
        let mut sequencer = SequencerActor::new(
            sequencer_key,
            rooch_db.rooch_store,
            ServiceStatus::Active,
            &registry_service.default_registry(),
            None,
        )?;
        assert_eq!(sequencer.last_order(), last_tx_order);
        for _ in 0..10 {
            let tx_data = LedgerTxData::L2Tx(RoochTransaction::mock());
            let ledger_tx = sequencer.sequence(tx_data)?;
            assert_eq!(ledger_tx.sequence_info.tx_order, last_tx_order + 1);
            last_tx_order = ledger_tx.sequence_info.tx_order;
        }
        assert_eq!(sequencer.last_order(), last_tx_order);

        let _ = store_instance.cancel_metrics_task();
        // Wait for rocksdb cancel metrics task to avoid db lock
        tokio::time::sleep(Duration::from_millis(CF_METRICS_REPORT_PERIOD_MILLIS)).await;
    }
    // load from db again
    {
        // To aviod AlreadyReg for re init the same db
        let new_registry = prometheus::Registry::new();
        let rooch_db = RoochDB::init(opt.store_config(), &new_registry)?;
        let sequencer_key = RoochKeyPair::generate_secp256k1();
        let mut sequencer = SequencerActor::new(
            sequencer_key,
            rooch_db.rooch_store,
            ServiceStatus::Active,
            &new_registry,
            None,
        )?;
        assert_eq!(sequencer.last_order(), last_tx_order);
        let tx_data = LedgerTxData::L2Tx(RoochTransaction::mock());
        let ledger_tx = sequencer.sequence(tx_data)?;
        assert_eq!(ledger_tx.sequence_info.tx_order, last_tx_order + 1);
    }
    Ok(())
}

// test concurrent
// Build a sequencer actor and sequence transactions concurrently
#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_sequencer_concurrent() -> Result<()> {
    let opt = RoochOpt::new_with_temp_store()?;
    let registry_service = RegistryService::default();
    let rooch_db = init_rooch_db(&opt, &registry_service.default_registry())?;
    let sequencer_key = RoochKeyPair::generate_secp256k1();

    let actor_system = ActorSystem::global_system();

    let sequencer = SequencerActor::new(
        sequencer_key,
        rooch_db.rooch_store,
        ServiceStatus::Active,
        &registry_service.default_registry(),
        None,
    )?
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

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use moveos_types::move_types::as_struct_tag;
use rooch_benchmarks::indexer::{
    gen_indexer_object_states_with_tx_order, prepare_indexer_object_states_with_tx_order,
};
use rooch_framework_tests::binding_test;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::store::traits::IndexerStoreTrait;
use rooch_indexer::IndexerStore;
use rooch_types::indexer::state::{IndexerObjectState, ObjectStateFilter};
use std::cell::RefCell;

fn bench_read_object_states(c: &mut Criterion) {
    let binding_test = binding_test::RustBindingTest::new_in_tokio().unwrap();
    let mut group = c.benchmark_group("read_object_states");

    let indexer_store = binding_test.rooch_db().indexer_store.clone();
    let indexer_reader = binding_test.rooch_db().indexer_reader.clone();

    // Because of genesis tx and subsequent txs, reserved tx order position start with tx_order = 90
    let states = prepare_indexer_object_states_with_tx_order(100_000_0, 90);
    bench_read_with_reader(
        &mut group,
        "indexer_read",
        indexer_store.clone(),
        indexer_reader.clone(),
        states.clone(),
    );

    group.sample_size(10);

    group.finish();
}

fn bench_read_with_reader(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    indexer_store: IndexerStore,
    indexer_reader: IndexerReader,
    states: Vec<IndexerObjectState>,
) {
    let states_len = states.len() as u64;
    indexer_store
        .persist_or_update_object_states(states.clone())
        .unwrap();

    group
        .bench_with_input(
            BenchmarkId::new(id, states_len),
            &(indexer_reader, states, states_len),
            |b, input| {
                let (indexer_reader, states, state_len) = input;
                let mut i = 0usize;
                b.iter_with_setup(
                    || {
                        let state = states[i % (*state_len as usize)].clone();
                        i += 1;
                        state
                    },
                    |state| {
                        let object_type =
                            as_struct_tag(state.metadata.object_type.clone()).unwrap();
                        let object_state_filter1 = ObjectStateFilter::ObjectTypeWithOwner {
                            object_type: object_type.clone(),
                            owner: state.metadata.owner,
                            filter_out: false,
                        };
                        let result1 = indexer_reader
                            .query_object_states_with_filter(object_state_filter1, None, 50, true)
                            .unwrap();
                        assert!(!result1.is_empty());
                        let object_state_filter2 = ObjectStateFilter::ObjectType(object_type);
                        let result2 = indexer_reader
                            .query_object_states_with_filter(object_state_filter2, None, 50, true)
                            .unwrap();
                        assert!(!result2.is_empty());
                    },
                );
            },
        )
        .sample_size(100);
}

fn bench_write_object_states(c: &mut Criterion) {
    let binding_test = binding_test::RustBindingTest::new_in_tokio().unwrap();
    let mut group = c.benchmark_group("write_object_states");

    let indexer_store = binding_test.rooch_db().indexer_store.clone();

    // Because of genesis tx and subsequent txs, reserved tx order position start with tx_order = 90
    let states = prepare_indexer_object_states_with_tx_order(100_000_0, 90);
    bench_write_with_store(
        &mut group,
        "indexer_write",
        indexer_store.clone(),
        states.clone(),
    );

    group.sample_size(10);

    group.finish();
}

fn bench_write_with_store(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    indexer_store: IndexerStore,
    states: Vec<IndexerObjectState>,
) {
    let states_len = states.len() as u64;
    indexer_store
        .persist_or_update_object_states(states)
        .unwrap();

    // Must skip tx_orders in prepare_indexer_object_states_with_tx_order, just from tx_oder = 100
    let mut tx_order_offset = RefCell::new(100u64);
    group
        .bench_with_input(
            BenchmarkId::new(id, states_len),
            &(indexer_store, states_len),
            |b, input| {
                let (indexer_store, _states_len) = input;

                let once_num: usize = 100;
                // Must skip tx_orders in prepare_indexer_object_states_with_tx_order, just from tx_oder = 100
                // let mut tx_order_offset = 100u64;
                b.iter_with_setup(
                    || {
                        // let new_tx_order = tx_order_offset;
                        // tx_order_offset += 1;
                        let new_tx_order = *tx_order_offset.borrow();
                        *tx_order_offset.borrow_mut() += 1;
                        gen_indexer_object_states_with_tx_order(once_num, new_tx_order)
                    },
                    |data| {
                        indexer_store.persist_or_update_object_states(data).unwrap();
                    },
                );
            },
        )
        .sample_size(100);
}

criterion_group!(benches, bench_write_object_states, bench_read_object_states);
criterion_main!(benches);

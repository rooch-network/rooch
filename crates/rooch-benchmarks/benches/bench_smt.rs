// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use ethers::types::H256;
use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
use rooch_benchmarks::smt::{gen_kv_from_seed, prepare_change_set, Blob};
use rooch_framework_tests::binding_test;
use smt::{InMemoryNodeStore, NodeReader, NodeWriter, SMTree, TreeChangeSet};

fn bench_get_with_proof(c: &mut Criterion) {
    let binding_test = binding_test::RustBindingTest::new().unwrap();

    let mut group = c.benchmark_group("get_with_proof");

    let mem_store = InMemoryNodeStore::default();
    let db_store = binding_test
        .rooch_db()
        .moveos_store
        .get_state_store()
        .node_store
        .clone();

    let (ks, changeset) = prepare_change_set(*GENESIS_STATE_ROOT, 100_000).unwrap();

    bench_get_with_proof_with_tree(
        &mut group,
        "mem_store",
        mem_store,
        ks.clone(),
        changeset.clone(),
    );

    bench_get_with_proof_with_tree(
        &mut group,
        "db_store",
        db_store.clone(),
        ks.clone(),
        changeset.clone(),
    );

    let (ks, changeset) = prepare_change_set(*GENESIS_STATE_ROOT, 1_000_000).unwrap();

    bench_get_with_proof_with_tree(&mut group, "db_store", db_store, ks, changeset);

    group.finish();
}

fn bench_get_with_proof_with_tree<NS>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    node_store: NS,
    ks: Vec<H256>,
    changeset: TreeChangeSet,
) where
    NS: NodeReader + NodeWriter + Clone + 'static,
{
    let tree: SMTree<H256, Blob, NS> = SMTree::new(node_store.clone());

    node_store.write_nodes(changeset.nodes.clone()).unwrap();
    let key_nums = ks.len();
    group
        .bench_with_input(
            BenchmarkId::new(id, key_nums),
            &(tree, changeset.state_root, ks),
            |b, input| {
                let (tree, state_root, ks) = input;
                let k_len = ks.len();
                let mut i = 0usize;
                b.iter_with_setup(
                    || {
                        let k = &ks[i % k_len];
                        i += 1;
                        k
                    },
                    |k| {
                        let (value, _proof) = tree.get_with_proof(*state_root, *k).unwrap();
                        assert!(value.is_some());
                    },
                );
            },
        )
        .sample_size(100);
}

fn bench_put_and_commit(c: &mut Criterion) {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let mut group = c.benchmark_group("put_and_commit");

    let mem_store = InMemoryNodeStore::default();
    let db_store = binding_test
        .rooch_db()
        .moveos_store
        .get_state_store()
        .node_store
        .clone();

    let (ks, changeset) = prepare_change_set(*GENESIS_STATE_ROOT, 100_000).unwrap();

    bench_put_with_tree(
        &mut group,
        "mem_store",
        mem_store,
        ks.len() as u64,
        changeset.clone(),
    );
    bench_put_with_tree(
        &mut group,
        "db_store",
        db_store.clone(),
        ks.len() as u64,
        changeset,
    );

    let (ks, changeset) = prepare_change_set(*GENESIS_STATE_ROOT, 1_000_000).unwrap();

    bench_put_with_tree(
        &mut group,
        "db_store",
        db_store.clone(),
        ks.len() as u64,
        changeset,
    );

    group.sample_size(100);

    group.finish();
}

fn bench_put_with_tree<NS>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    node_store: NS,
    key_nums: u64,
    changeset: TreeChangeSet,
) where
    NS: NodeReader + NodeWriter + Clone + 'static,
{
    let tree: SMTree<H256, Blob, NS> = SMTree::new(node_store.clone());

    node_store.write_nodes(changeset.nodes.clone()).unwrap();
    group
        .bench_with_input(
            BenchmarkId::new(id, key_nums),
            &(tree, changeset.state_root, node_store),
            |b, input| {
                let (tree, state_root, node_store) = input;
                let seed = *state_root;
                let mut state_root = *state_root;
                let k_len = 10;
                b.iter_with_setup(
                    || gen_kv_from_seed(seed, k_len),
                    |kvs| {
                        let changeset = tree.puts(state_root, kvs).unwrap();
                        state_root = changeset.state_root;
                        node_store.write_nodes(changeset.nodes.clone()).unwrap();
                    },
                );
            },
        )
        .sample_size(100);
}

criterion_group!(benches, bench_get_with_proof, bench_put_and_commit);
criterion_main!(benches);

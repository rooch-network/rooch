// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::log::ExecutionAndIOCosts;
use crate::log::ExecutionGasEvent;
use crate::render::Render;
use move_core_types::gas_algebra::{GasQuantity, InternalGas};
use std::collections::{btree_map, BTreeMap};

/// Represents an aggregation of execution gas events, including the count and total gas costs for each type of event.
///
/// The events are sorted by the amount of gas used, from high to low.
#[derive(Debug)]
pub struct AggregatedExecutionGasEvents {
    pub ops: Vec<(String, usize, InternalGas)>,
}

fn insert_or_add<K, U>(
    map: &mut BTreeMap<K, (usize, GasQuantity<U>)>,
    key: K,
    amount: GasQuantity<U>,
) where
    K: Ord,
{
    if amount.is_zero() {
        return;
    }
    match map.entry(key) {
        btree_map::Entry::Occupied(entry) => {
            let r = entry.into_mut();
            r.0 += 1;
            r.1 += amount;
        }
        btree_map::Entry::Vacant(entry) => {
            entry.insert((1, amount));
        }
    }
}

fn into_sorted_vec<I, K, N>(collection: I) -> Vec<(K, usize, N)>
where
    N: Ord,
    I: IntoIterator<Item = (K, (usize, N))>,
{
    let mut v = collection
        .into_iter()
        .map(|(key, (count, amount))| (key, count, amount))
        .collect::<Vec<_>>();
    // Sort in descending order.
    v.sort_by(|(_key1, _count1, amount1), (_key2, _count2, amount2)| amount2.cmp(amount1));
    v
}

impl ExecutionAndIOCosts {
    /// Counts the number of hits and aggregates the gas costs for each type of event.
    pub fn aggregate_gas_events(&self) -> AggregatedExecutionGasEvents {
        use ExecutionGasEvent::*;

        let mut ops = BTreeMap::new();
        let mut storage_reads = BTreeMap::new();

        for event in self.gas_events() {
            match event {
                Loc(..) | Call(..) => (),
                Bytecode { op, cost } => insert_or_add(
                    &mut ops,
                    format!("{:?}", op).to_ascii_lowercase().to_string(),
                    *cost,
                ),
                CallNative {
                    module_id,
                    fn_name,
                    ty_args,
                    cost,
                } => insert_or_add(
                    &mut ops,
                    format!(
                        "{}",
                        Render(&(module_id, fn_name.as_ident_str(), ty_args.as_slice())),
                    ),
                    *cost,
                ),
                LoadResource {
                    addr: _addr,
                    ty,
                    cost,
                } => insert_or_add(&mut storage_reads, format!("{}", ty), *cost),
                CreateTy { cost } => insert_or_add(&mut ops, "create_ty".to_string(), *cost),
            }
        }

        AggregatedExecutionGasEvents {
            ops: into_sorted_vec(ops),
        }
    }
}

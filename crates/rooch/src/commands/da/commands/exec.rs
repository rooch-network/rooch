// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{collect_chunks, get_tx_list_from_chunk};
use bitcoin::hashes::Hash;
use bitcoin_client::proxy::BitcoinClientProxy;
use clap::Parser;

use moveos_types::h256::H256;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;

use rooch_executor::proxy::ExecutorProxy;
use rooch_types::bitcoin::types::Block as BitcoinBlock;
use rooch_types::transaction::{L1BlockWithBody, LedgerTransaction, LedgerTxData};

/// exec LedgerTransaction List for verification.
#[derive(Debug, Parser)]
pub struct ExecCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(
        long = "order-state-path",
        help = "Path to tx_order:state_root file(results from RoochNetwork), for verification"
    )]
    pub order_state_path: PathBuf,
}

impl ExecCommand {
    pub fn execute(self) -> anyhow::Result<()> {
        let executor = self.build_exec_inner();
        executor.execute()?;
        Ok(())
    }

    fn build_exec_inner(&self) -> ExecInner {
        let (order_state_pair, tx_order_end) = self.load_order_state_pair();
        let chunks = collect_chunks(self.segment_dir.clone()).unwrap();
        ExecInner {
            segment_dir: self.segment_dir.clone(),
            chunks,
            order_state_pair,
            tx_order_end,
            bitcoin_client_proxy: None,
            executor: ExecutorProxy {},
        }
    }

    fn load_order_state_pair(&self) -> (HashMap<u64, H256>, u64) {
        let mut order_state_pair = HashMap::new();
        let mut tx_order_end = 0;

        let mut reader = BufReader::new(File::open(self.order_state_path.clone()).unwrap());
        // collect all tx_order:state_root pairs
        for line in reader.by_ref().lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split(':').collect();
            let tx_order = parts[0].parse::<u64>().unwrap();
            let state_root = H256::from_str(parts[1]).unwrap();
            order_state_pair.insert(tx_order, state_root);
            if tx_order > tx_order_end {
                tx_order_end = tx_order;
            }
        }
        (order_state_pair, tx_order_end)
    }
}

struct ExecInner {
    segment_dir: PathBuf,
    chunks: HashMap<u128, Vec<u64>>,
    order_state_pair: HashMap<u64, H256>,
    tx_order_end: u64,

    bitcoin_client_proxy: Option<BitcoinClientProxy>,
    pub(crate) executor: ExecutorProxy,
}

impl ExecInner {
    async fn execute_verify(&self) -> anyhow::Result<()> {
        let mut block_number = 0;
        let mut max_verified_tx_order = 0;
        // TODO two thread: one produce tx, one consume tx
        loop {
            let tx_list = self.load_ledger_tx_list(block_number)?;
            if tx_list.is_none() {
                break;
            }
            let tx_list = tx_list.unwrap();
            for ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                if tx_order > self.tx_order_end {
                    break;
                }
                self.execute_verify_tx(ledger_tx)?;
                max_verified_tx_order = tx_order;
            }
            block_number += 1;
        }
        println!(
            "All transactions execution state root are strictly equal to RoochNetwork: [0, {}]",
            max_verified_tx_order
        );
        Ok(())
    }

    fn load_ledger_tx_list(
        &self,
        block_number: u128,
    ) -> anyhow::Result<Option<Vec<LedgerTransaction>>> {
        let segments = self.chunks.get(&block_number);
        if segments.is_none() {
            return Ok(None);
        }
        let tx_list = get_tx_list_from_chunk(
            self.segment_dir.clone(),
            block_number,
            segments.unwrap().clone(),
        )?;
        Ok(Some(tx_list))
    }

    async fn execute_verify_tx(&self, ledger_tx: LedgerTransaction) -> anyhow::Result<()> {
        let tx_order = ledger_tx.sequence_info.tx_order;
        if tx_order > self.tx_order_end {
            return Ok(());
        }

        match &ledger_tx.data {
            LedgerTxData::L1Block(block) => match &self.bitcoin_client_proxy {
                Some(bitcoin_client_proxy) => {
                    let block_hash_vec = block.block_hash.clone();
                    let block_hash = bitcoin::block::BlockHash::from_slice(&block_hash_vec)?;
                    let btc_block = bitcoin_client_proxy.get_block(block_hash).await?;
                    let block_body = BitcoinBlock::from(btc_block);
                    let moveos_tx = self
                        .executor
                        .validate_l1_block(L1BlockWithBody::new(block.clone(), block_body.encode()))
                        .await?;
                    self.execute_tx(ledger_tx.clone(), moveos_tx).await?;
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "The bitcoin client proxy should be initialized (block: {:?} needed)",
                        block
                    ));
                }
            },
            LedgerTxData::L1Tx(l1_tx) => {
                let moveos_tx = self.executor.validate_l1_tx(l1_tx.clone()).await?;
                self.execute_tx(ledger_tx.clone(), moveos_tx).await?;
            }
            LedgerTxData::L2Tx(l2_tx) => {
                let moveos_tx = self.executor.validate_l2_tx(l2_tx.clone()).await?;
                self.execute_tx(ledger_tx.clone(), moveos_tx).await?;
            }
        }
        Ok(())
    }

    async fn execute_tx(
        &self,
        tx: LedgerTransaction,
        mut moveos_tx: VerifiedMoveOSTransaction,
    ) -> anyhow::Result<()> {
        let tx_order = tx.sequence_info.tx_order;
        moveos_tx.ctx.add(tx.sequence_info.clone())?;
        let (_output, execution_info) =
            self.executor.execute_transaction(moveos_tx.clone()).await?;

        let root = execution_info.root_metadata();
        let expected_root_opt = self.order_state_pair.get(&tx_order);
        match expected_root_opt {
            Some(expected_root) => {
                if root.state_root.unwrap() != *expected_root {
                    return Err(anyhow::anyhow!(
                        "Execution state root is not equal to RoochNetwork: tx_order: {}, exp: {:?}, act: {:?}",
                        tx.sequence_info.tx_order,
                        *expected_root, root
                    ));
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}

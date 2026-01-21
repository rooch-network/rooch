// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use anyhow::{bail, Result};
use bitcoin::{Address, Amount};
use moveos_types::moveos_std::object::{ObjectID, GENESIS_STATE_ROOT};
use rooch_rpc_api::jsonrpc_types::{
    btc::utxo::{UTXOFilterView, UTXOObjectView, UTXOStateView},
    IndexerStateIDView,
};
use rooch_rpc_client::Client;
use rooch_types::bitcoin::{types::OutPoint, utxo::derive_utxo_id};
use tokio::time::Duration;
use tracing::debug;

// Retry configuration for handling rate limiting (HTTP 429)
// Fixed delay strategy: wait 2 seconds between retries
// Based on rate limit: 10 req/ms with 200 burst size, refills in ~20ms
const MAX_RETRIES: u32 = 20;
const RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub struct UTXOSelector {
    client: Client,
    sender: Address,
    specific_utxos: Vec<ObjectID>,
    loaded_page: Option<(Option<IndexerStateIDView>, bool)>,
    candidate_utxos: VecDeque<UTXOObjectView>,
    skip_seal_check: bool,
}

impl UTXOSelector {
    pub async fn new(
        client: Client,
        sender: Address,
        specific_utxos: Vec<ObjectID>,
        skip_seal_check: bool,
    ) -> Result<Self> {
        let mut selector = Self {
            client,
            sender,
            specific_utxos,
            loaded_page: None,
            candidate_utxos: VecDeque::new(),
            skip_seal_check,
        };
        selector.load_specific_utxos().await?;
        Ok(selector)
    }

    /// Create a UTXOSelector with pre-loaded UTXOs, avoiding redundant queries
    pub fn with_utxos(client: Client, sender: Address, utxos: Vec<UTXOObjectView>) -> Self {
        // Convert to VecDeque so pop_back returns items in order (oldest first)
        let mut candidate_utxos = VecDeque::new();
        for utxo in utxos {
            // We use push_front so that pop_back will return items in FIFO order
            // The oldest UTXO should be spent first to avoid bad-txns-premature-spend-of-coinbase
            candidate_utxos.push_front(utxo);
        }

        Self {
            client,
            sender,
            specific_utxos: vec![],
            loaded_page: Some((None, false)), // Mark as no more pages to load
            candidate_utxos,
            skip_seal_check: true, // Already checked when loading
        }
    }

    async fn load_specific_utxos(&mut self) -> Result<()> {
        if self.specific_utxos.is_empty() {
            return Ok(());
        }

        // RPC has a limit of 100 object IDs per request
        const BATCH_SIZE: usize = 100;
        let minimal_non_dust = self.sender.script_pubkey().minimal_non_dust();

        for chunk in self.specific_utxos.chunks(BATCH_SIZE) {
            let utxos_objs = self
                .client
                .rooch
                .query_utxos(UTXOFilterView::object_ids(chunk.to_vec()), None, None, None)
                .await?;

            for utxo_state_view in utxos_objs.data {
                let utxo = &utxo_state_view.value;
                if !self.skip_seal_check {
                    if skip_utxo(&utxo_state_view, minimal_non_dust) {
                        bail!("UTXO {} has seal or tempstate attachment: {:?}, please use --skip-seal-check to skip this check", utxo_state_view.value.outpoint(), utxo_state_view);
                    }
                }
                if utxo_state_view.metadata.owner_bitcoin_address.is_none() {
                    bail!(
                        "Can not recognize the owner of UTXO {}, metadata: {:?}",
                        utxo.outpoint(),
                        utxo_state_view.metadata
                    );
                }
                self.candidate_utxos.push_front(utxo_state_view.into());
            }
        }
        Ok(())
    }

    async fn load_utxos(&mut self) -> Result<()> {
        let (next_cursor, has_next_page) = self.loaded_page.unwrap_or((None, true));
        if !has_next_page {
            return Ok(());
        }

        // Retry loop with fixed delay for rate limiting
        let mut retry_count = 0;

        loop {
            let result = self
                .client
                .rooch
                .query_utxos(
                    UTXOFilterView::owner(self.sender.clone()),
                    next_cursor.map(Into::into),
                    None,
                    Some(false),
                )
                .await;

            match result {
                Ok(utxo_page) => {
                    debug!("loaded utxos: {:?}", utxo_page.data.len());
                    let minimal_non_dust = self.sender.script_pubkey().minimal_non_dust();
                    for utxo_view in utxo_page.data {
                        let utxo = &utxo_view.value;
                        if !self.skip_seal_check && skip_utxo(&utxo_view, minimal_non_dust) {
                            continue;
                        }
                        if utxo_view.metadata.owner_bitcoin_address.is_none() {
                            debug!(
                                "Can not recognize the owner of UTXO {}, metadata: {:?}, skip.",
                                utxo.outpoint(),
                                utxo_view.metadata
                            );
                            continue;
                        }
                        // We use deque to make sure the utxos are popped in the order they are loaded, the oldest utxo will be popped first
                        // Avoid bad-txns-premature-spend-of-coinbase error
                        self.candidate_utxos.push_front(utxo_view.into());
                    }
                    self.loaded_page = Some((utxo_page.next_cursor, utxo_page.has_next_page));
                    return Ok(());
                }
                Err(e) if is_rate_limit_error(&e) && retry_count < MAX_RETRIES => {
                    retry_count += 1;
                    debug!(
                        "Rate limited while loading UTXOs (attempt {}/{}), retrying after {:?}",
                        retry_count, MAX_RETRIES, RETRY_DELAY
                    );
                    tokio::time::sleep(RETRY_DELAY).await;
                }
                Err(e) => {
                    // Either not a rate limit error or max retries exceeded
                    return Err(e.context(format!(
                        "Failed to load UTXOs after {} retries",
                        retry_count
                    )));
                }
            }
        }
    }

    /// Get the next utxo from the candidate utxos
    pub async fn next_utxo(&mut self) -> Result<Option<UTXOObjectView>> {
        if self.candidate_utxos.is_empty() {
            debug!("candidate_utxos is empty, loading more UTXOs...");
            self.load_utxos().await?;
            debug!(
                "After loading, candidate_utxos count: {}",
                self.candidate_utxos.len()
            );
        }
        Ok(self.candidate_utxos.pop_back())
    }

    pub async fn select_utxos(&mut self, expected_amount: Amount) -> Result<Vec<UTXOObjectView>> {
        let mut utxos = vec![];
        let mut total_input = Amount::from_sat(0);
        let mut iteration_count = 0;
        debug!(
            "select_utxos: expected_amount={} satoshi",
            expected_amount.to_sat()
        );
        while total_input < expected_amount {
            iteration_count += 1;
            let utxo = self.next_utxo().await?;
            if utxo.is_none() {
                debug!(
                    "select_utxos: No more UTXOs after {} iterations. total_input={} satoshi, expected={}",
                    iteration_count,
                    total_input.to_sat(),
                    expected_amount.to_sat()
                );
                bail!("not enough BTC funds");
            }
            let utxo = utxo.unwrap();
            total_input += utxo.amount();
            utxos.push(utxo.clone());
            debug!(
                "Iteration {}: UTXO {} added, amount={}, total_input={}",
                iteration_count,
                utxo.outpoint(),
                utxo.amount().to_sat(),
                total_input.to_sat()
            );
        }
        debug!(
            "select_utxos: Successfully selected {} UTXOs totaling {} satoshi",
            utxos.len(),
            total_input.to_sat()
        );
        Ok(utxos)
    }

    /// Load all UTXOs for the sender address and return them
    /// This is used when we need to know the total count of UTXOs before building transactions
    pub async fn load_all_utxos(&mut self) -> Result<Vec<UTXOObjectView>> {
        let mut all_utxos = Vec::new();

        // Keep loading pages until no more
        loop {
            self.load_utxos().await?;
            if self.candidate_utxos.is_empty() {
                break;
            }
            // Collect all currently loaded UTXOs
            while let Some(utxo) = self.candidate_utxos.pop_back() {
                all_utxos.push(utxo);
            }

            // Check if there's a next page
            let (_, has_next_page) = self.loaded_page.unwrap_or((None, false));
            if !has_next_page {
                break;
            }
        }

        debug!("load_all_utxos: Loaded {} UTXOs", all_utxos.len());
        Ok(all_utxos)
    }

    pub fn specific_utxos(&self) -> &[ObjectID] {
        &self.specific_utxos
    }

    pub async fn get_utxo(&self, outpoint: &OutPoint) -> Result<UTXOObjectView> {
        let utxo_obj_id = derive_utxo_id(outpoint);
        self.client
            .rooch
            .get_utxo_object(utxo_obj_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("UTXO {} not found", outpoint))
    }
}

fn skip_utxo(utxo_state_view: &UTXOStateView, minimal_non_dust: Amount) -> bool {
    let utxo = &utxo_state_view.value;
    if !utxo.seals.is_empty() {
        debug!(
            "UTXO {} is has seals: {:?}, skip.",
            utxo.outpoint(),
            utxo.seals
        );
        return true;
    }
    if utxo.amount() <= minimal_non_dust {
        debug!(
            "UTXO {} is less than dust value: {}, skip.",
            utxo.outpoint(),
            minimal_non_dust
        );
        return true;
    }
    if utxo_state_view.metadata.state_root.is_some()
        && utxo_state_view.metadata.state_root.as_ref().unwrap().0 != *GENESIS_STATE_ROOT
    {
        debug!("UTXO {} is contains tempstate, skip.", utxo.outpoint());
        return true;
    }
    false
}

fn is_rate_limit_error(error: &anyhow::Error) -> bool {
    let error_msg = error.to_string().to_lowercase();
    // Check for various rate limit indicators
    error_msg.contains("too many requests")
        || error_msg.contains("429")
        || error_msg.contains("serverisbusy")
        || error_msg.contains("wait for")
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use bitcoin::{Address, Amount};
use moveos_types::moveos_std::object::{ObjectID, GENESIS_STATE_ROOT};
use rooch_rpc_api::jsonrpc_types::{
    btc::utxo::{UTXOFilterView, UTXOStateView, UTXOView},
    IndexerStateIDView, ObjectMetaView,
};
use rooch_rpc_client::Client;
use tracing::debug;

#[derive(Debug)]
pub struct UTXOSelector {
    client: Client,
    sender: Address,
    specific_utxos: Vec<ObjectID>,
    loaded_page: Option<(Option<IndexerStateIDView>, bool)>,
    candidate_utxos: Vec<(ObjectMetaView, UTXOView)>,
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
            candidate_utxos: vec![],
            skip_seal_check,
        };
        selector.load_specific_utxos().await?;
        Ok(selector)
    }

    async fn load_specific_utxos(&mut self) -> Result<()> {
        let utxos_objs = self
            .client
            .rooch
            .query_utxos(
                UTXOFilterView::object_ids(self.specific_utxos.clone()),
                None,
                None,
                None,
            )
            .await?;
        for utxo_state_view in utxos_objs.data {
            let utxo = &utxo_state_view.value;
            if !self.skip_seal_check {
                let minimal_non_dust = self.sender.script_pubkey().minimal_non_dust();
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
            self.candidate_utxos
                .push((utxo_state_view.metadata, utxo_state_view.value));
        }
        Ok(())
    }

    async fn load_utxos(&mut self) -> Result<()> {
        let (next_cursor, has_next_page) = self.loaded_page.unwrap_or((None, true));
        if !has_next_page {
            return Ok(());
        }
        let utxo_page = self
            .client
            .rooch
            .query_utxos(
                UTXOFilterView::owner(self.sender.clone()),
                next_cursor.map(Into::into),
                None,
                None,
            )
            .await?;
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
            self.candidate_utxos
                .push((utxo_view.metadata, utxo_view.value));
        }
        self.loaded_page = Some((utxo_page.next_cursor, utxo_page.has_next_page));
        Ok(())
    }
    /// Get the next utxo from the candidate utxos
    pub async fn next_utxo(&mut self) -> Result<Option<(ObjectMetaView, UTXOView)>> {
        if self.candidate_utxos.is_empty() {
            self.load_utxos().await?;
        }
        Ok(self.candidate_utxos.pop())
    }

    pub fn specific_utxos(&self) -> &[ObjectID] {
        &self.specific_utxos
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

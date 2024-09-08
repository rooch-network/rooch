// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use bitcoin::Address;
use moveos_types::{moveos_std::object::ObjectID, state::ObjectState};
use rooch_rpc_api::jsonrpc_types::{btc::utxo::UTXOFilterView, IndexerStateIDView};
use rooch_rpc_client::Client;
use rooch_types::bitcoin::utxo::UTXO;
use tracing::debug;

#[derive(Debug)]
pub struct UTXOSelector {
    client: Client,
    sender: Address,
    specific_utxos: Vec<ObjectID>,
    loaded_page: Option<(Option<IndexerStateIDView>, bool)>,
    candidate_utxos: Vec<UTXO>,
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
            .get_object_states(self.specific_utxos.clone(), None)
            .await?;
        for (utxo_id, utxo_obj) in self.specific_utxos.iter().zip(utxos_objs) {
            if let Some(utxo_obj_view) = utxo_obj {
                let utxo_obj: ObjectState = utxo_obj_view.into();
                let utxo = utxo_obj.value_as::<UTXO>()?;
                if !self.skip_seal_check {
                    if !utxo.seals.is_empty() {
                        bail!("UTXO {} is has seals: {:?}, please use --skip-seal-check to skip this check", utxo.outpoint(), utxo.seals);
                    }
                    let dust_value = self.sender.script_pubkey().dust_value();
                    if utxo.amount() <= dust_value {
                        bail!("UTXO {} is less than dust value: {}, please use --skip-dust-check to skip this check", utxo.outpoint(), dust_value);
                    }
                }
                self.candidate_utxos.push(utxo);
            } else {
                bail!("UTXO not found: {}", utxo_id);
            }
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
        let dust_value = self.sender.script_pubkey().dust_value();
        for utxo_view in utxo_page.data {
            let utxo: UTXO = utxo_view.value.into();
            if !self.skip_seal_check {
                if !utxo.seals.is_empty() {
                    continue;
                }
                if utxo.amount() <= dust_value {
                    continue;
                }
            }
            self.candidate_utxos.push(utxo);
        }
        self.loaded_page = Some((utxo_page.next_cursor, utxo_page.has_next_page));
        Ok(())
    }
    /// Get the next utxo from the candidate utxos
    pub async fn next_utxo(&mut self) -> Result<Option<UTXO>> {
        if self.candidate_utxos.is_empty() {
            self.load_utxos().await?;
        }
        Ok(self.candidate_utxos.pop())
    }

    pub fn specific_utxos(&self) -> &[ObjectID] {
        &self.specific_utxos
    }
}

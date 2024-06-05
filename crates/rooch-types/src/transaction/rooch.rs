// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::RawTransaction;
use super::{authenticator::Authenticator, AuthenticatorInfo};
use crate::address::RoochAddress;
use crate::rooch_network::BuiltinChainID;
use anyhow::Result;
use moveos_types::h256::H256;
use moveos_types::moveos_std::gas_schedule::GasScheduleConfig;
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::{
    moveos_std::tx_context::TxContext,
    transaction::{MoveAction, MoveOSTransaction},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoochTransactionData {
    /// Sender's address.
    pub sender: RoochAddress,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
    // The ChainID of the transaction.
    pub chain_id: u64,
    // The max gas to be used.
    pub max_gas_amount: u64,
    // The MoveAction to execute.
    pub action: MoveAction,
}

impl RoochTransactionData {
    pub fn new(
        sender: RoochAddress,
        sequence_number: u64,
        chain_id: u64,
        max_gas_amount: u64,
        action: MoveAction,
    ) -> Self {
        Self {
            sender,
            sequence_number,
            chain_id,
            max_gas_amount,
            action,
        }
    }

    pub fn new_for_test(sender: RoochAddress, sequence_number: u64, action: MoveAction) -> Self {
        Self {
            sender,
            sequence_number,
            chain_id: BuiltinChainID::Local.chain_id().id(),
            max_gas_amount: GasScheduleConfig::INITIAL_MAX_GAS_AMOUNT,
            action,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn tx_hash(&self) -> H256 {
        moveos_types::h256::sha3_256_of(self.encode().as_slice())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoochTransaction {
    data: RoochTransactionData,
    authenticator: Authenticator,

    #[serde(skip_serializing, skip_deserializing)]
    data_hash: Option<H256>,
}

impl RoochTransaction {
    pub fn new(data: RoochTransactionData, authenticator: Authenticator) -> Self {
        Self {
            data,
            authenticator,
            data_hash: None,
        }
    }

    pub fn new_genesis_tx(
        genesis_address: RoochAddress,
        chain_id: u64,
        action: MoveAction,
    ) -> Self {
        Self {
            data: RoochTransactionData::new(genesis_address, 0, chain_id, u64::max_value(), action),
            authenticator: Authenticator::genesis(),
            data_hash: None,
        }
    }

    pub fn sender(&self) -> RoochAddress {
        self.data.sender
    }

    pub fn sequence_number(&self) -> u64 {
        self.data.sequence_number
    }

    pub fn chain_id(&self) -> u64 {
        self.data.chain_id
    }

    pub fn max_gas_amount(&self) -> u64 {
        self.data.max_gas_amount
    }

    pub fn action(&self) -> &MoveAction {
        &self.data.action
    }

    pub fn decode(bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        bcs::from_bytes::<Self>(bytes).map_err(Into::into)
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn tx_hash(&mut self) -> H256 {
        if let Some(hash) = self.data_hash {
            hash
        } else {
            let hash = self.data.tx_hash();
            self.data_hash = Some(hash);
            self.data_hash.unwrap()
        }
    }

    pub fn authenticator_info(&self) -> AuthenticatorInfo {
        AuthenticatorInfo::new(self.chain_id(), self.authenticator.clone())
    }

    pub fn authenticator(&self) -> &Authenticator {
        &self.authenticator
    }

    pub fn tx_size(&self) -> u64 {
        bcs::serialized_size(self).expect("serialize transaction size should success") as u64
    }

    //TODO use protest Arbitrary to generate mock data
    pub fn mock() -> RoochTransaction {
        use crate::{address::RoochSupportedAddress, crypto::RoochKeyPair};
        use move_core_types::{
            account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
        };
        use moveos_types::move_types::FunctionId;

        let sender: RoochAddress = RoochAddress::random();
        let sequence_number = 0;
        let payload = MoveAction::new_function_call(
            FunctionId::new(
                ModuleId::new(AccountAddress::random(), Identifier::new("test").unwrap()),
                Identifier::new("test").unwrap(),
            ),
            vec![],
            vec![],
        );

        let transaction_data = RoochTransactionData::new_for_test(sender, sequence_number, payload);

        let kp = &RoochKeyPair::generate_secp256k1();
        let auth = Authenticator::bitcoin(kp, &transaction_data);

        RoochTransaction::new(transaction_data, auth)
    }

    pub fn into_moveos_transaction(mut self, root: RootObjectEntity) -> MoveOSTransaction {
        let tx_hash = self.tx_hash();
        let tx_size = self.tx_size();
        let tx_ctx = TxContext::new(
            self.data.sender.into(),
            self.data.sequence_number,
            self.data.max_gas_amount,
            tx_hash,
            tx_size,
        );
        MoveOSTransaction::new(root, tx_ctx, self.data.action)
    }
}

impl TryFrom<RawTransaction> for RoochTransaction {
    type Error = anyhow::Error;

    fn try_from(raw: RawTransaction) -> Result<Self> {
        let tx = RoochTransaction::decode(raw.raw.as_slice())?;
        Ok(tx)
    }
}

use parking_lot::RwLock;
use rooch_types::transaction::TypedTransaction;
use rooch_types::H256;
use std::sync::Arc;

pub trait TxStore {
    fn add(&mut self, transaction: TypedTransaction);
    fn get_by_hash(&self, hash: H256) -> Option<TypedTransaction>;
    fn get_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction>;
}

pub struct TxDB {
    tx_store: InMemoryStore,
}

impl TxDB {
    pub fn new_with_memory_store() -> Self {
        Self {
            tx_store: InMemoryStore::default(),
        }
    }

    pub fn add(&mut self, transaction: TypedTransaction) {
        self.tx_store.add(transaction);
    }

    pub fn get_by_hash(&self, hash: H256) -> Option<TypedTransaction> {
        self.tx_store.get_by_hash(hash)
    }

    pub fn get_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction> {
        self.tx_store.get_by_index(start, limit)
    }
}

#[derive(Default, Clone)]
pub struct InMemoryStore {
    inner: Arc<RwLock<Vec<TypedTransaction>>>,
}

impl TxStore for InMemoryStore {
    fn add(&mut self, transaction: TypedTransaction) {
        let mut inner = self.inner.write();
        inner.push(transaction);
    }

    fn get_by_hash(&self, hash: H256) -> Option<TypedTransaction> {
        let inner = self.inner.read();
        inner.iter().find(|tx| tx.hash() == hash).cloned()
    }

    fn get_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction> {
        let inner = self.inner.read();
        let end = std::cmp::min((start + limit) as usize, inner.len());
        inner[start as usize..end].to_vec()
    }
}

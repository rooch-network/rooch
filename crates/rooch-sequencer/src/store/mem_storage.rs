use crate::storage::storage::TransactionStorage;
use rooch_types::transaction::TypedTransaction;
use rooch_types::H256;

pub struct TransactionMemoryStorage {
    head: Option<Box<MemTransaction>>,
}

impl Default for TransactionMemoryStorage {
    fn default() -> Self {
        Self { head: None }
    }
}

struct MemTransaction {
    pub tx: TypedTransaction,
    pub next: Option<Box<MemTransaction>>,
}

impl TransactionStorage for TransactionMemoryStorage {
    fn add_transaction(&mut self, transaction: TypedTransaction) {
        let mut new_node = Box::new(MemTransaction {
            tx: transaction,
            next: None,
        });
        if let Some(head) = self.head.take() {
            new_node.next = Some(head);
        }
        self.head = Some(new_node);
    }

    fn get_transaction_by_hash(&self, hash: H256) -> Option<TypedTransaction> {
        let mut current = self.head.as_ref();
        while let Some(node) = current {
            if node.tx.hash() == hash {
                return Some(node.tx.clone());
            }
            current = node.next.as_ref();
        }
        None
    }

    fn get_transaction_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction> {
        let mut current = self.head.as_ref();
        let mut count = 0;
        let mut result = Vec::new();
        while let Some(node) = current {
            if count >= start && (result.len() as u64) < limit {
                result.push(node.tx.clone());
            }
            current = node.next.as_ref();
            count += 1;
        }
        result
    }
}

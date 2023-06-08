use rooch_types::transaction::TypedTransaction;
use rooch_types::H256;

pub trait TransactionStorage {
    fn add(&mut self, transaction: TypedTransaction);
    fn get_by_hash(&self, hash: H256) -> Option<TypedTransaction>;
    fn get_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction>;
}

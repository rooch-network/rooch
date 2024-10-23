use std::{
    collections::HashMap,
    rc::{Rc, Weak},
    cell::RefCell,
};

use cosmwasm_std::{Order, Record};
use cosmwasm_vm::{BackendError, BackendResult, GasInfo, Storage};
use moveos_types::moveos_std::object::ObjectID;

pub struct ProxyStorage {
    storages: HashMap<ObjectID, Weak<RefCell<Box<dyn Storage>>>>,
}

impl ProxyStorage {
    pub fn new() -> Self {
        ProxyStorage {
            storages: HashMap::new(),
        }
    }

    pub fn register(&mut self, prefix: ObjectID, storage: Rc<RefCell<Box<dyn Storage>>>) {
        self.storages.insert(prefix, Rc::downgrade(&storage));
    }

    pub fn unregister(&mut self, prefix: &ObjectID) -> Option<Rc<RefCell<Box<dyn Storage>>>> {
        self.storages.remove(prefix).and_then(|weak| weak.upgrade())
    }

    fn get_storage(&self, key: &[u8]) -> Result<Rc<RefCell<Box<dyn Storage>>>, BackendError> {
        let key_str = std::str::from_utf8(key)
            .map_err(|e| BackendError::Unknown { msg: format!("Invalid UTF-8 sequence: {}", e) })?;
        
        let parts: Vec<&str> = key_str.split('/').collect();
        if parts.len() < 2 {
            return Err(BackendError::Unknown { msg: "Invalid key format".to_string() });
        }

        let object_id = ObjectID::from_hex_literal(parts[1])
            .map_err(|e| BackendError::Unknown { msg: e.to_string() })?;

        self.storages.get(&object_id)
            .and_then(|weak| weak.upgrade())
            .ok_or_else(|| BackendError::Unknown {
                msg: format!("No storage found or storage dropped for object ID: {:?}", object_id),
            })
    }
}

impl Storage for ProxyStorage {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        match self.get_storage(key) {
            Ok(storage_rc) => storage_rc.borrow().get(key),
            Err(e) => (Err(e), GasInfo::new(0, 0)),
        }
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        match self.get_storage(key) {
            Ok(storage_rc) => {
                storage_rc.borrow_mut().set(key, value)
            },
            Err(e) => (Err(e), GasInfo::new(0, 0)),
        }
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        match self.get_storage(key) {
            Ok(storage_rc) => {
                storage_rc.borrow_mut().remove(key)
            },
            Err(e) => (Err(e), GasInfo::new(0, 0)),
        }
    }

    fn scan(&mut self, start: Option<&[u8]>, end: Option<&[u8]>, order: Order) -> BackendResult<u32> {
        let start = match start {
            Some(s) => s,
            None => return (
                Err(BackendError::Unknown {
                    msg: "Scan without start key is not supported".to_string(),
                }),
                GasInfo::new(0, 0)
            ),
        };
    
        match self.get_storage(start) {
            Ok(storage_rc) => {
                storage_rc.borrow_mut().scan(Some(start), end, order)
            },
            Err(e) => (Err(e), GasInfo::new(0, 0)),
        }
    }
  
    fn next(&mut self, _iterator_id: u32) -> BackendResult<Option<Record>> {
        (Err(BackendError::Unknown {
            msg: "ProxyStorage does not support iterators directly".to_string(),
        }), GasInfo::new(0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockStorage {
        data: HashMap<Vec<u8>, Vec<u8>>,
    }

    impl MockStorage {
        fn new() -> Self {
            MockStorage {
                data: HashMap::new(),
            }
        }
    }

    impl Storage for MockStorage {
        fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
            (Ok(self.data.get(key).cloned()), GasInfo::new(1, 0))
        }

        fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
            self.data.insert(key.to_vec(), value.to_vec());
            (Ok(()), GasInfo::new(1, 0))
        }

        fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
            self.data.remove(key);
            (Ok(()), GasInfo::new(1, 0))
        }

        fn scan(&mut self, _start: Option<&[u8]>, _end: Option<&[u8]>, _order: Order) -> BackendResult<u32> {
            (Ok(0), GasInfo::new(1, 0))
        }

        fn next(&mut self, _iterator_id: u32) -> BackendResult<Option<Record>> {
            (Ok(None), GasInfo::new(1, 0))
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut proxy = ProxyStorage::new();
        let obj_id = ObjectID::random();
        let key = format!("/{}/key", obj_id.to_hex());
        let value = b"test_value".to_vec();

        let storage = Rc::new(RefCell::new(Box::new(MockStorage::new()) as Box<dyn Storage>));
        {
            let mut storage_mut = storage.borrow_mut();
            storage_mut.set(key.as_bytes(), &value).0.unwrap();
        }

        proxy.register(obj_id, Rc::clone(&storage));

        let (result, _) = proxy.get(key.as_bytes());
        assert_eq!(result.unwrap(), Some(value));
    }

    #[test]
    fn test_unregister() {
        let mut proxy = ProxyStorage::new();
        let obj_id = ObjectID::random();
        let storage = Rc::new(RefCell::new(Box::new(MockStorage::new()) as Box<dyn Storage>));

        proxy.register(obj_id.clone(), Rc::clone(&storage));
        assert!(proxy.unregister(&obj_id).is_some());
        assert!(proxy.unregister(&obj_id).is_none());
    }

    #[test]
    fn test_set_and_remove() {
        let mut proxy = ProxyStorage::new();
        let obj_id = ObjectID::random();
        let key = format!("/{}/key", obj_id.to_hex());
        let value = b"test_value".to_vec();

        let storage = Rc::new(RefCell::new(Box::new(MockStorage::new()) as Box<dyn Storage>));
        proxy.register(obj_id, Rc::clone(&storage));

        proxy.set(key.as_bytes(), &value).0.unwrap();
        let (get_result, _) = proxy.get(key.as_bytes());
        assert_eq!(get_result.unwrap(), Some(value));

        proxy.remove(key.as_bytes()).0.unwrap();
        let (get_result, _) = proxy.get(key.as_bytes());
        assert_eq!(get_result.unwrap(), None);
    }

    #[test]
    fn test_nonexistent_storage() {
        let proxy = ProxyStorage::new();
        let obj_id = ObjectID::random();
        let key = obj_id.to_hex();

        let (result, _) = proxy.get(key.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_object_id() {
        let proxy = ProxyStorage::new();
        let key = vec![0; 31]; // Invalid length for ObjectID

        let (result, _) = proxy.get(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan() {
        let mut proxy = ProxyStorage::new();
        let obj_id = ObjectID::random();
        let start_key = format!("/{}/start", obj_id.to_hex());
        let end_key = format!("/{}/end", obj_id.to_hex());

        let storage = Rc::new(RefCell::new(Box::new(MockStorage::new()) as Box<dyn Storage>));
        proxy.register(obj_id, Rc::clone(&storage));

        let (result, _) = proxy.scan(Some(start_key.as_bytes()), Some(end_key.as_bytes()), Order::Ascending);
        assert!(result.is_ok());
    }

    #[test]
    fn test_next() {
        let mut proxy = ProxyStorage::new();
        
        let (result, _) = proxy.next(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_dropped() {
        let mut proxy = ProxyStorage::new();
        let obj_id = ObjectID::random();
        let key = format!("/{}/key", obj_id.to_hex());
        let value = b"test_value".to_vec();

        {
            // Create a scoped storage
            let storage = Rc::new(RefCell::new(Box::new(MockStorage::new()) as Box<dyn Storage>));
            {
                let mut storage_mut = storage.borrow_mut();
                storage_mut.set(key.as_bytes(), &value).0.unwrap();
            }
            proxy.register(obj_id.clone(), Rc::clone(&storage));
            // `storage` is dropped here
        }

        // Now, the storage has been dropped, and ProxyStorage should detect it
        let (result, _) = proxy.get(key.as_bytes());
        assert!(result.is_err());
    }
}


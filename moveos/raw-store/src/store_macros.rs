// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
macro_rules! derive_store {
    ($store_type: ident, $key_type: ty, $value_type: ty, $prefix_name: expr) => {
        #[derive(Clone)]
        pub struct $store_type {
            store: $crate::InnerStore<Self>,
        }

        impl $store_type {
            pub fn new(instance: $crate::StoreInstance) -> Self {
                Self {
                    store: $crate::InnerStore::new(instance),
                }
            }
        }

        impl $crate::ColumnFamily for $store_type {
            type Key = $key_type;
            type Value = $value_type;

            fn name() -> $crate::ColumnFamilyName {
                $prefix_name
            }
        }

        impl $crate::SchemaStore for $store_type {
            fn get_store(&self) -> &$crate::InnerStore<Self> {
                &self.store
            }
        }
    };
}

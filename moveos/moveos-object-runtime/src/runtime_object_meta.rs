// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    account_address::AccountAddress, language_storage::TypeTag, value::MoveTypeLayout,
    vm_status::StatusCode,
};
use moveos_types::{
    h256::H256,
    moveos_std::object::{ObjectID, ObjectMeta, SYSTEM_OWNER_ADDRESS},
};

#[derive(Debug, Clone)]
pub(crate) enum DataStatus {
    Clean,
    Dirty,
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectMetaValue {
    metadata: ObjectMeta,
    value_layout: MoveTypeLayout,
    status: DataStatus,
}

pub(crate) enum RuntimeObjectMeta {
    None(ObjectID),
    Fresh(ObjectMetaValue),
    Cached(ObjectMetaValue),
    Deleted(ObjectMetaValue),
}

impl RuntimeObjectMeta {
    pub fn none(obj_id: ObjectID) -> Self {
        RuntimeObjectMeta::None(obj_id)
    }

    pub fn cached(metadata: ObjectMeta, value_layout: MoveTypeLayout) -> Self {
        RuntimeObjectMeta::Cached(ObjectMetaValue {
            metadata,
            value_layout,
            status: DataStatus::Clean,
        })
    }

    pub fn init(
        &mut self,
        value_type: TypeTag,
        value_layout: MoveTypeLayout,
    ) -> PartialVMResult<()> {
        let obj_id = match self {
            Self::None(id) => id.clone(),
            //If the object is removed, and init it again, we treat it is fresh
            Self::Deleted(meta) => meta.metadata.id.clone(),
            Self::Fresh(v) | Self::Cached(v) => {
                //If the object is removed, and init it again, the value type may be different
                v.metadata.value_type = value_type;
                v.value_layout = value_layout;
                return Ok(());
            }
        };

        let metadata = ObjectMeta::genesis_meta(obj_id, value_type);
        *self = Self::Fresh(ObjectMetaValue {
            metadata,
            value_layout,
            status: DataStatus::Dirty,
        });
        Ok(())
    }

    pub fn move_from(&mut self) -> PartialVMResult<ObjectMeta> {
        match self {
            Self::None(id) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("ObjectMeta {} is None", id))),
            Self::Fresh(meta) => {
                let id = meta.metadata.id.clone();
                match std::mem::replace(self, Self::None(id)) {
                    Self::Fresh(meta) => Ok(meta.metadata),
                    _ => unreachable!(),
                }
            }
            Self::Cached(meta) => {
                let meta_copy = meta.clone();
                match std::mem::replace(self, Self::Deleted(meta_copy)) {
                    Self::Cached(meta) => Ok(meta.metadata),
                    _ => unreachable!(),
                }
            }
            Self::Deleted(meta) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("ObjectMeta {} is Deleted", meta.metadata.id))),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, RuntimeObjectMeta::None(_))
    }

    pub fn id(&self) -> &ObjectID {
        match self {
            Self::None(id) => id,
            Self::Deleted(meta) => &meta.metadata.id,
            Self::Fresh(meta) => &meta.metadata.id,
            Self::Cached(meta) => &meta.metadata.id,
        }
    }

    pub fn metadata(&self) -> PartialVMResult<&ObjectMeta> {
        match self {
            Self::None(id) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("ObjectMeta of {} is None", id))),
            Self::Deleted(meta) => {
                //We allow get read-only metadata of deleted object
                Ok(&meta.metadata)
            }
            Self::Fresh(meta) => Ok(&meta.metadata),
            Self::Cached(meta) => Ok(&meta.metadata),
        }
    }

    pub fn value_type(&self) -> PartialVMResult<&TypeTag> {
        let meta = self.metadata()?;
        Ok(&meta.value_type)
    }

    pub fn value_layout(&self) -> PartialVMResult<&MoveTypeLayout> {
        match self {
            Self::None(id) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("Layout of {} is None or Deleted", id))),
            Self::Deleted(meta) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("Layout of {} is Deleted", &meta.metadata.id))),
            Self::Fresh(meta) => Ok(&meta.value_layout),
            Self::Cached(meta) => Ok(&meta.value_layout),
        }
    }

    pub fn state_root(&self) -> PartialVMResult<H256> {
        let meta = self.metadata()?;
        Ok(meta.state_root())
    }

    fn metadata_mut(&mut self) -> PartialVMResult<&mut ObjectMeta> {
        let meta_value = match self {
            RuntimeObjectMeta::None(id) => {
                return Err(PartialVMError::new(StatusCode::MISSING_DATA)
                    .with_message(format!("ObjectMeta of {} is None", id)))
            }
            RuntimeObjectMeta::Deleted(meta) => {
                debug_assert!(false, "Deleted object should not be modified");
                return Err(PartialVMError::new(StatusCode::MISSING_DATA)
                    .with_message(format!("ObjectMeta of {} is Deleted", &meta.metadata.id)));
            }
            RuntimeObjectMeta::Fresh(meta) => meta,
            RuntimeObjectMeta::Cached(meta) => meta,
        };
        meta_value.status = DataStatus::Dirty;
        Ok(&mut meta_value.metadata)
    }

    pub fn transfer(&mut self, new_owner: AccountAddress) -> PartialVMResult<()> {
        let meta = self.metadata_mut()?;
        meta.owner = new_owner;
        Ok(())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_frozen(&mut self) -> PartialVMResult<()> {
        let meta = self.metadata_mut()?;
        meta.to_frozen();
        Ok(())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_shared(&mut self) -> PartialVMResult<()> {
        let meta = self.metadata_mut()?;
        meta.to_shared();
        Ok(())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_system_owner(&mut self) -> PartialVMResult<()> {
        let meta = self.metadata_mut()?;
        meta.owner = SYSTEM_OWNER_ADDRESS;
        Ok(())
    }

    pub fn increase_size(&mut self) -> PartialVMResult<u64> {
        let meta = self.metadata_mut()?;
        match meta.size.checked_add(1) {
            Some(size) => {
                meta.size = size;
                if log::log_enabled!(log::Level::Trace) {
                    tracing::trace!(
                        object_id = tracing::field::display(&meta.id),
                        op = "increase_size",
                        "Object {} increase size to {}",
                        &meta.id,
                        size
                    );
                }
                Ok(size)
            }
            None => Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!("Object {} size overflow, this should not happen", self.id()),
                ),
            ),
        }
    }

    pub fn decrease_size(&mut self) -> PartialVMResult<u64> {
        let meta = self.metadata_mut()?;
        match meta.size.checked_sub(1) {
            Some(size) => {
                meta.size = size;
                if log::log_enabled!(log::Level::Trace) {
                    tracing::trace!(
                        object_id = tracing::field::display(&meta.id),
                        op = "decrease_size",
                        "Object {} decrease size to {}",
                        &meta.id,
                        size
                    );
                }
                Ok(size)
            }
            None => Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!(
                        "Object {} size underflow, this should not happen",
                        self.id()
                    ),
                ),
            ),
        }
    }

    pub fn update_timestamp(&mut self, timestamp: u64) -> PartialVMResult<()> {
        let meta = self.metadata_mut()?;
        if meta.created_at == 0 {
            meta.created_at = timestamp;
        }
        meta.updated_at = timestamp;
        Ok(())
    }

    pub fn into_effect(self) -> Option<(ObjectMeta, bool)> {
        match self {
            RuntimeObjectMeta::None(_) => None,
            RuntimeObjectMeta::Fresh(meta) => Some((meta.metadata, true)),
            RuntimeObjectMeta::Cached(meta) => match meta.status {
                DataStatus::Clean => Some((meta.metadata, false)),
                DataStatus::Dirty => Some((meta.metadata, true)),
            },
            RuntimeObjectMeta::Deleted(meta) => Some((meta.metadata, true)),
        }
    }
}

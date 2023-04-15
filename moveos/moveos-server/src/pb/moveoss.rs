#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag = "2")]
    pub version: u64,
    #[prost(message, optional, tag = "3")]
    pub info: ::core::option::Option<TransactionInfo>,
    #[prost(uint64, tag = "4")]
    pub epoch: u64,
    #[prost(uint64, tag = "5")]
    pub block_height: u64,
    #[prost(enumeration = "transaction::TransactionType", tag = "6")]
    pub r#type: i32,
    #[prost(oneof = "transaction::TxnData", tags = "7, 8, 9, 10")]
    pub txn_data: ::core::option::Option<transaction::TxnData>,
}
/// Nested message and enum types in `Transaction`.
pub mod transaction {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum TransactionType {
        Unspecified = 0,
        Genesis = 1,
        BlockMetadata = 2,
        StateCheckpoint = 3,
        User = 4,
    }
    impl TransactionType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                TransactionType::Unspecified => "TRANSACTION_TYPE_UNSPECIFIED",
                TransactionType::Genesis => "TRANSACTION_TYPE_GENESIS",
                TransactionType::BlockMetadata => "TRANSACTION_TYPE_BLOCK_METADATA",
                TransactionType::StateCheckpoint => "TRANSACTION_TYPE_STATE_CHECKPOINT",
                TransactionType::User => "TRANSACTION_TYPE_USER",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TRANSACTION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "TRANSACTION_TYPE_GENESIS" => Some(Self::Genesis),
                "TRANSACTION_TYPE_BLOCK_METADATA" => Some(Self::BlockMetadata),
                "TRANSACTION_TYPE_STATE_CHECKPOINT" => Some(Self::StateCheckpoint),
                "TRANSACTION_TYPE_USER" => Some(Self::User),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum TxnData {
        #[prost(message, tag = "7")]
        BlockMetadata(super::BlockMetadataTransaction),
        #[prost(message, tag = "8")]
        Genesis(super::GenesisTransaction),
        #[prost(message, tag = "9")]
        StateCheckpoint(super::StateCheckpointTransaction),
        #[prost(message, tag = "10")]
        User(super::UserTransaction),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMetadataTransaction {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub round: u64,
    #[prost(message, repeated, tag = "3")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(bytes = "vec", tag = "4")]
    pub previous_block_votes_bitvec: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "5")]
    pub proposer: ::prost::alloc::string::String,
    #[prost(uint32, repeated, tag = "6")]
    pub failed_proposer_indices: ::prost::alloc::vec::Vec<u32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisTransaction {
    #[prost(message, optional, tag = "1")]
    pub payload: ::core::option::Option<WriteSet>,
    #[prost(message, repeated, tag = "2")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateCheckpointTransaction {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserTransaction {
    #[prost(message, optional, tag = "1")]
    pub request: ::core::option::Option<UserTransactionRequest>,
    #[prost(message, repeated, tag = "2")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<EventKey>,
    #[prost(uint64, tag = "2")]
    pub sequence_number: u64,
    #[prost(message, optional, tag = "3")]
    pub r#type: ::core::option::Option<MoveType>,
    #[prost(string, tag = "5")]
    pub type_str: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub data: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub state_change_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub event_root_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub state_checkpoint_hash: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint64, tag = "5")]
    pub gas_used: u64,
    #[prost(bool, tag = "6")]
    pub success: bool,
    #[prost(string, tag = "7")]
    pub vm_status: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "8")]
    pub accumulator_root_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "9")]
    pub changes: ::prost::alloc::vec::Vec<WriteSetChange>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventKey {
    #[prost(uint64, tag = "1")]
    pub creation_number: u64,
    #[prost(string, tag = "2")]
    pub account_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserTransactionRequest {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub sequence_number: u64,
    #[prost(uint64, tag = "3")]
    pub max_gas_amount: u64,
    #[prost(uint64, tag = "4")]
    pub gas_unit_price: u64,
    #[prost(message, optional, tag = "5")]
    pub expiration_timestamp_secs: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "6")]
    pub payload: ::core::option::Option<TransactionPayload>,
    #[prost(message, optional, tag = "7")]
    pub signature: ::core::option::Option<Signature>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteSet {
    #[prost(enumeration = "write_set::WriteSetType", tag = "1")]
    pub write_set_type: i32,
    #[prost(oneof = "write_set::WriteSet", tags = "2, 3")]
    pub write_set: ::core::option::Option<write_set::WriteSet>,
}
/// Nested message and enum types in `WriteSet`.
pub mod write_set {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum WriteSetType {
        Unspecified = 0,
        ScriptWriteSet = 1,
        DirectWriteSet = 2,
    }
    impl WriteSetType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                WriteSetType::Unspecified => "WRITE_SET_TYPE_UNSPECIFIED",
                WriteSetType::ScriptWriteSet => "WRITE_SET_TYPE_SCRIPT_WRITE_SET",
                WriteSetType::DirectWriteSet => "WRITE_SET_TYPE_DIRECT_WRITE_SET",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "WRITE_SET_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "WRITE_SET_TYPE_SCRIPT_WRITE_SET" => Some(Self::ScriptWriteSet),
                "WRITE_SET_TYPE_DIRECT_WRITE_SET" => Some(Self::DirectWriteSet),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum WriteSet {
        #[prost(message, tag = "2")]
        ScriptWriteSet(super::ScriptWriteSet),
        #[prost(message, tag = "3")]
        DirectWriteSet(super::DirectWriteSet),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScriptWriteSet {
    #[prost(string, tag = "1")]
    pub execute_as: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub script: ::core::option::Option<ScriptPayload>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DirectWriteSet {
    #[prost(message, repeated, tag = "1")]
    pub write_set_change: ::prost::alloc::vec::Vec<WriteSetChange>,
    #[prost(message, repeated, tag = "2")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteSetChange {
    #[prost(enumeration = "write_set_change::Type", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "write_set_change::Change", tags = "2, 3, 4, 5, 6, 7")]
    pub change: ::core::option::Option<write_set_change::Change>,
}
/// Nested message and enum types in `WriteSetChange`.
pub mod write_set_change {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unspecified = 0,
        DeleteModule = 1,
        DeleteResource = 2,
        DeleteTableItem = 3,
        WriteModule = 4,
        WriteResource = 5,
        WriteTableItem = 6,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Unspecified => "TYPE_UNSPECIFIED",
                Type::DeleteModule => "TYPE_DELETE_MODULE",
                Type::DeleteResource => "TYPE_DELETE_RESOURCE",
                Type::DeleteTableItem => "TYPE_DELETE_TABLE_ITEM",
                Type::WriteModule => "TYPE_WRITE_MODULE",
                Type::WriteResource => "TYPE_WRITE_RESOURCE",
                Type::WriteTableItem => "TYPE_WRITE_TABLE_ITEM",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "TYPE_DELETE_MODULE" => Some(Self::DeleteModule),
                "TYPE_DELETE_RESOURCE" => Some(Self::DeleteResource),
                "TYPE_DELETE_TABLE_ITEM" => Some(Self::DeleteTableItem),
                "TYPE_WRITE_MODULE" => Some(Self::WriteModule),
                "TYPE_WRITE_RESOURCE" => Some(Self::WriteResource),
                "TYPE_WRITE_TABLE_ITEM" => Some(Self::WriteTableItem),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Change {
        #[prost(message, tag = "2")]
        DeleteModule(super::DeleteModule),
        #[prost(message, tag = "3")]
        DeleteResource(super::DeleteResource),
        #[prost(message, tag = "4")]
        DeleteTableItem(super::DeleteTableItem),
        #[prost(message, tag = "5")]
        WriteModule(super::WriteModule),
        #[prost(message, tag = "6")]
        WriteResource(super::WriteResource),
        #[prost(message, tag = "7")]
        WriteTableItem(super::WriteTableItem),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteModule {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub state_key_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub module: ::core::option::Option<MoveModuleId>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResource {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub state_key_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub r#type: ::core::option::Option<MoveStructTag>,
    #[prost(string, tag = "4")]
    pub type_str: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTableItem {
    #[prost(bytes = "vec", tag = "1")]
    pub state_key_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub handle: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub key: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub data: ::core::option::Option<DeleteTableData>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTableData {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub key_type: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteModule {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub state_key_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub data: ::core::option::Option<MoveModuleBytecode>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteResource {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub state_key_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub r#type: ::core::option::Option<MoveStructTag>,
    #[prost(string, tag = "4")]
    pub type_str: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub data: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteTableData {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub key_type: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub value: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub value_type: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteTableItem {
    #[prost(bytes = "vec", tag = "1")]
    pub state_key_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub handle: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub key: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub data: ::core::option::Option<WriteTableData>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPayload {
    #[prost(enumeration = "transaction_payload::Type", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "transaction_payload::Payload", tags = "2, 3, 4, 5, 6")]
    pub payload: ::core::option::Option<transaction_payload::Payload>,
}
/// Nested message and enum types in `TransactionPayload`.
pub mod transaction_payload {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unspecified = 0,
        EntryFunctionPayload = 1,
        ScriptPayload = 2,
        ModuleBundlePayload = 3,
        WriteSetPayload = 4,
        MultisigPayload = 5,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Unspecified => "TYPE_UNSPECIFIED",
                Type::EntryFunctionPayload => "TYPE_ENTRY_FUNCTION_PAYLOAD",
                Type::ScriptPayload => "TYPE_SCRIPT_PAYLOAD",
                Type::ModuleBundlePayload => "TYPE_MODULE_BUNDLE_PAYLOAD",
                Type::WriteSetPayload => "TYPE_WRITE_SET_PAYLOAD",
                Type::MultisigPayload => "TYPE_MULTISIG_PAYLOAD",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "TYPE_ENTRY_FUNCTION_PAYLOAD" => Some(Self::EntryFunctionPayload),
                "TYPE_SCRIPT_PAYLOAD" => Some(Self::ScriptPayload),
                "TYPE_MODULE_BUNDLE_PAYLOAD" => Some(Self::ModuleBundlePayload),
                "TYPE_WRITE_SET_PAYLOAD" => Some(Self::WriteSetPayload),
                "TYPE_MULTISIG_PAYLOAD" => Some(Self::MultisigPayload),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "2")]
        EntryFunctionPayload(super::EntryFunctionPayload),
        #[prost(message, tag = "3")]
        ScriptPayload(super::ScriptPayload),
        #[prost(message, tag = "4")]
        ModuleBundlePayload(super::ModuleBundlePayload),
        #[prost(message, tag = "5")]
        WriteSetPayload(super::WriteSetPayload),
        #[prost(message, tag = "6")]
        MultisigPayload(super::MultisigPayload),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntryFunctionPayload {
    #[prost(message, optional, tag = "1")]
    pub function: ::core::option::Option<EntryFunctionId>,
    #[prost(message, repeated, tag = "2")]
    pub type_arguments: ::prost::alloc::vec::Vec<MoveType>,
    #[prost(string, repeated, tag = "3")]
    pub arguments: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag = "4")]
    pub entry_function_id_str: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveScriptBytecode {
    #[prost(bytes = "vec", tag = "1")]
    pub bytecode: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub abi: ::core::option::Option<MoveFunction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScriptPayload {
    #[prost(message, optional, tag = "1")]
    pub code: ::core::option::Option<MoveScriptBytecode>,
    #[prost(message, repeated, tag = "2")]
    pub type_arguments: ::prost::alloc::vec::Vec<MoveType>,
    #[prost(string, repeated, tag = "3")]
    pub arguments: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MultisigPayload {
    #[prost(string, tag = "1")]
    pub multisig_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub transaction_payload: ::core::option::Option<MultisigTransactionPayload>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MultisigTransactionPayload {
    #[prost(enumeration = "multisig_transaction_payload::Type", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "multisig_transaction_payload::Payload", tags = "2")]
    pub payload: ::core::option::Option<multisig_transaction_payload::Payload>,
}
/// Nested message and enum types in `MultisigTransactionPayload`.
pub mod multisig_transaction_payload {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unspecified = 0,
        EntryFunctionPayload = 1,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Unspecified => "TYPE_UNSPECIFIED",
                Type::EntryFunctionPayload => "TYPE_ENTRY_FUNCTION_PAYLOAD",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "TYPE_ENTRY_FUNCTION_PAYLOAD" => Some(Self::EntryFunctionPayload),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "2")]
        EntryFunctionPayload(super::EntryFunctionPayload),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleBundlePayload {
    #[prost(message, repeated, tag = "1")]
    pub modules: ::prost::alloc::vec::Vec<MoveModuleBytecode>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveModuleBytecode {
    #[prost(bytes = "vec", tag = "1")]
    pub bytecode: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub abi: ::core::option::Option<MoveModule>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveModule {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub friends: ::prost::alloc::vec::Vec<MoveModuleId>,
    #[prost(message, repeated, tag = "4")]
    pub exposed_functions: ::prost::alloc::vec::Vec<MoveFunction>,
    #[prost(message, repeated, tag = "5")]
    pub structs: ::prost::alloc::vec::Vec<MoveStruct>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveFunction {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(enumeration = "move_function::Visibility", tag = "2")]
    pub visibility: i32,
    #[prost(bool, tag = "3")]
    pub is_entry: bool,
    #[prost(message, repeated, tag = "4")]
    pub generic_type_params: ::prost::alloc::vec::Vec<MoveFunctionGenericTypeParam>,
    #[prost(message, repeated, tag = "5")]
    pub params: ::prost::alloc::vec::Vec<MoveType>,
    #[prost(message, repeated, tag = "6")]
    pub r#return: ::prost::alloc::vec::Vec<MoveType>,
}
/// Nested message and enum types in `MoveFunction`.
pub mod move_function {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Visibility {
        Unspecified = 0,
        Private = 1,
        Public = 2,
        Friend = 3,
    }
    impl Visibility {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Visibility::Unspecified => "VISIBILITY_UNSPECIFIED",
                Visibility::Private => "VISIBILITY_PRIVATE",
                Visibility::Public => "VISIBILITY_PUBLIC",
                Visibility::Friend => "VISIBILITY_FRIEND",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "VISIBILITY_UNSPECIFIED" => Some(Self::Unspecified),
                "VISIBILITY_PRIVATE" => Some(Self::Private),
                "VISIBILITY_PUBLIC" => Some(Self::Public),
                "VISIBILITY_FRIEND" => Some(Self::Friend),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveStruct {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub is_native: bool,
    #[prost(enumeration = "MoveAbility", repeated, tag = "3")]
    pub abilities: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, repeated, tag = "4")]
    pub generic_type_params: ::prost::alloc::vec::Vec<MoveStructGenericTypeParam>,
    #[prost(message, repeated, tag = "5")]
    pub fields: ::prost::alloc::vec::Vec<MoveStructField>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveStructGenericTypeParam {
    #[prost(enumeration = "MoveAbility", repeated, tag = "1")]
    pub constraints: ::prost::alloc::vec::Vec<i32>,
    #[prost(bool, tag = "2")]
    pub is_phantom: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveStructField {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub r#type: ::core::option::Option<MoveType>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveFunctionGenericTypeParam {
    #[prost(enumeration = "MoveAbility", repeated, tag = "1")]
    pub constraints: ::prost::alloc::vec::Vec<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveType {
    #[prost(enumeration = "MoveTypes", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "move_type::Content", tags = "3, 4, 5, 6, 7")]
    pub content: ::core::option::Option<move_type::Content>,
}
/// Nested message and enum types in `MoveType`.
pub mod move_type {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ReferenceType {
        #[prost(bool, tag = "1")]
        pub mutable: bool,
        #[prost(message, optional, boxed, tag = "2")]
        pub to: ::core::option::Option<::prost::alloc::boxed::Box<super::MoveType>>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Content {
        #[prost(message, tag = "3")]
        Vector(::prost::alloc::boxed::Box<super::MoveType>),
        #[prost(message, tag = "4")]
        Struct(super::MoveStructTag),
        #[prost(uint32, tag = "5")]
        GenericTypeParamIndex(u32),
        #[prost(message, tag = "6")]
        Reference(::prost::alloc::boxed::Box<ReferenceType>),
        #[prost(string, tag = "7")]
        Unparsable(::prost::alloc::string::String),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteSetPayload {
    #[prost(message, optional, tag = "1")]
    pub write_set: ::core::option::Option<WriteSet>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntryFunctionId {
    #[prost(message, optional, tag = "1")]
    pub module: ::core::option::Option<MoveModuleId>,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveModuleId {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveStructTag {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub module: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub generic_type_params: ::prost::alloc::vec::Vec<MoveType>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Signature {
    #[prost(enumeration = "signature::Type", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "signature::Signature", tags = "2, 3, 4")]
    pub signature: ::core::option::Option<signature::Signature>,
}
/// Nested message and enum types in `Signature`.
pub mod signature {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unspecified = 0,
        Ed25519 = 1,
        MultiEd25519 = 2,
        MultiAgent = 3,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Unspecified => "TYPE_UNSPECIFIED",
                Type::Ed25519 => "TYPE_ED25519",
                Type::MultiEd25519 => "TYPE_MULTI_ED25519",
                Type::MultiAgent => "TYPE_MULTI_AGENT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "TYPE_ED25519" => Some(Self::Ed25519),
                "TYPE_MULTI_ED25519" => Some(Self::MultiEd25519),
                "TYPE_MULTI_AGENT" => Some(Self::MultiAgent),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Signature {
        #[prost(message, tag = "2")]
        Ed25519(super::Ed25519Signature),
        #[prost(message, tag = "3")]
        MultiEd25519(super::MultiEd25519Signature),
        #[prost(message, tag = "4")]
        MultiAgent(super::MultiAgentSignature),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ed25519Signature {
    #[prost(bytes = "vec", tag = "1")]
    pub public_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MultiEd25519Signature {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub public_keys: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub signatures: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint32, tag = "3")]
    pub threshold: u32,
    #[prost(uint32, repeated, tag = "4")]
    pub public_key_indices: ::prost::alloc::vec::Vec<u32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MultiAgentSignature {
    #[prost(message, optional, tag = "1")]
    pub sender: ::core::option::Option<AccountSignature>,
    #[prost(string, repeated, tag = "2")]
    pub secondary_signer_addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "3")]
    pub secondary_signers: ::prost::alloc::vec::Vec<AccountSignature>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountSignature {
    #[prost(enumeration = "account_signature::Type", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "account_signature::Signature", tags = "2, 3")]
    pub signature: ::core::option::Option<account_signature::Signature>,
}
/// Nested message and enum types in `AccountSignature`.
pub mod account_signature {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unspecified = 0,
        Ed25519 = 1,
        MultiEd25519 = 2,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Unspecified => "TYPE_UNSPECIFIED",
                Type::Ed25519 => "TYPE_ED25519",
                Type::MultiEd25519 => "TYPE_MULTI_ED25519",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "TYPE_ED25519" => Some(Self::Ed25519),
                "TYPE_MULTI_ED25519" => Some(Self::MultiEd25519),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Signature {
        #[prost(message, tag = "2")]
        Ed25519(super::Ed25519Signature),
        #[prost(message, tag = "3")]
        MultiEd25519(super::MultiEd25519Signature),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactoinResponse {
    #[prost(string, tag = "1")]
    pub confirmation: ::prost::alloc::string::String,
}
/// for tests
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloResponse {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<::prost_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublishPackageRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub module: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublishPackageResponse {
    #[prost(string, tag = "1")]
    pub resp: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutionFunctionRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub functions: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutionFunctionResponse {
    #[prost(string, tag = "1")]
    pub resp: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MoveTypes {
    Unspecified = 0,
    Bool = 1,
    U8 = 2,
    U16 = 12,
    U32 = 13,
    U64 = 3,
    U128 = 4,
    U256 = 14,
    Address = 5,
    Signer = 6,
    /// `{ items: Box<MoveType> }`,
    Vector = 7,
    /// `(MoveStructTag)`,
    Struct = 8,
    /// `{ index: u16 }``,
    GenericTypeParam = 9,
    /// `{ mutable: bool, to: Box<MoveType> }`,
    Reference = 10,
    /// `(String)`,
    Unparsable = 11,
}
impl MoveTypes {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MoveTypes::Unspecified => "MOVE_TYPES_UNSPECIFIED",
            MoveTypes::Bool => "MOVE_TYPES_BOOL",
            MoveTypes::U8 => "MOVE_TYPES_U8",
            MoveTypes::U16 => "MOVE_TYPES_U16",
            MoveTypes::U32 => "MOVE_TYPES_U32",
            MoveTypes::U64 => "MOVE_TYPES_U64",
            MoveTypes::U128 => "MOVE_TYPES_U128",
            MoveTypes::U256 => "MOVE_TYPES_U256",
            MoveTypes::Address => "MOVE_TYPES_ADDRESS",
            MoveTypes::Signer => "MOVE_TYPES_SIGNER",
            MoveTypes::Vector => "MOVE_TYPES_VECTOR",
            MoveTypes::Struct => "MOVE_TYPES_STRUCT",
            MoveTypes::GenericTypeParam => "MOVE_TYPES_GENERIC_TYPE_PARAM",
            MoveTypes::Reference => "MOVE_TYPES_REFERENCE",
            MoveTypes::Unparsable => "MOVE_TYPES_UNPARSABLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MOVE_TYPES_UNSPECIFIED" => Some(Self::Unspecified),
            "MOVE_TYPES_BOOL" => Some(Self::Bool),
            "MOVE_TYPES_U8" => Some(Self::U8),
            "MOVE_TYPES_U16" => Some(Self::U16),
            "MOVE_TYPES_U32" => Some(Self::U32),
            "MOVE_TYPES_U64" => Some(Self::U64),
            "MOVE_TYPES_U128" => Some(Self::U128),
            "MOVE_TYPES_U256" => Some(Self::U256),
            "MOVE_TYPES_ADDRESS" => Some(Self::Address),
            "MOVE_TYPES_SIGNER" => Some(Self::Signer),
            "MOVE_TYPES_VECTOR" => Some(Self::Vector),
            "MOVE_TYPES_STRUCT" => Some(Self::Struct),
            "MOVE_TYPES_GENERIC_TYPE_PARAM" => Some(Self::GenericTypeParam),
            "MOVE_TYPES_REFERENCE" => Some(Self::Reference),
            "MOVE_TYPES_UNPARSABLE" => Some(Self::Unparsable),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MoveAbility {
    Unspecified = 0,
    Copy = 1,
    Drop = 2,
    Store = 3,
    Key = 4,
}
impl MoveAbility {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MoveAbility::Unspecified => "MOVE_ABILITY_UNSPECIFIED",
            MoveAbility::Copy => "MOVE_ABILITY_COPY",
            MoveAbility::Drop => "MOVE_ABILITY_DROP",
            MoveAbility::Store => "MOVE_ABILITY_STORE",
            MoveAbility::Key => "MOVE_ABILITY_KEY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MOVE_ABILITY_UNSPECIFIED" => Some(Self::Unspecified),
            "MOVE_ABILITY_COPY" => Some(Self::Copy),
            "MOVE_ABILITY_DROP" => Some(Self::Drop),
            "MOVE_ABILITY_STORE" => Some(Self::Store),
            "MOVE_ABILITY_KEY" => Some(Self::Key),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod os_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct OsServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl OsServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> OsServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> OsServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            OsServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn echo(
            &mut self,
            request: impl tonic::IntoRequest<super::HelloRequest>,
        ) -> Result<tonic::Response<super::HelloResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/moveoss.OsService/echo");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn publish(
            &mut self,
            request: impl tonic::IntoRequest<super::PublishPackageRequest>,
        ) -> Result<tonic::Response<super::PublishPackageResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/moveoss.OsService/publish");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn execute_function(
            &mut self,
            request: impl tonic::IntoRequest<super::ExecutionFunctionRequest>,
        ) -> Result<tonic::Response<super::ExecutionFunctionResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/moveoss.OsService/execute_function");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod os_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with OsServiceServer.
    #[async_trait]
    pub trait OsService: Send + Sync + 'static {
        async fn echo(
            &self,
            request: tonic::Request<super::HelloRequest>,
        ) -> Result<tonic::Response<super::HelloResponse>, tonic::Status>;
        async fn publish(
            &self,
            request: tonic::Request<super::PublishPackageRequest>,
        ) -> Result<tonic::Response<super::PublishPackageResponse>, tonic::Status>;
        async fn execute_function(
            &self,
            request: tonic::Request<super::ExecutionFunctionRequest>,
        ) -> Result<tonic::Response<super::ExecutionFunctionResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct OsServiceServer<T: OsService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: OsService> OsServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for OsServiceServer<T>
    where
        T: OsService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/moveoss.OsService/echo" => {
                    #[allow(non_camel_case_types)]
                    struct echoSvc<T: OsService>(pub Arc<T>);
                    impl<T: OsService> tonic::server::UnaryService<super::HelloRequest> for echoSvc<T> {
                        type Response = super::HelloResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::HelloRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = echoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/moveoss.OsService/publish" => {
                    #[allow(non_camel_case_types)]
                    struct publishSvc<T: OsService>(pub Arc<T>);
                    impl<T: OsService> tonic::server::UnaryService<super::PublishPackageRequest> for publishSvc<T> {
                        type Response = super::PublishPackageResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PublishPackageRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).publish(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = publishSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/moveoss.OsService/execute_function" => {
                    #[allow(non_camel_case_types)]
                    struct execute_functionSvc<T: OsService>(pub Arc<T>);
                    impl<T: OsService> tonic::server::UnaryService<super::ExecutionFunctionRequest>
                        for execute_functionSvc<T>
                    {
                        type Response = super::ExecutionFunctionResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ExecutionFunctionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).execute_function(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = execute_functionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: OsService> Clone for OsServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: OsService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: OsService> tonic::server::NamedService for OsServiceServer<T> {
        const NAME: &'static str = "moveoss.OsService";
    }
}

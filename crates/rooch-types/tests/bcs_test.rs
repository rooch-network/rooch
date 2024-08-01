use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId, u256::U256,
};
use moveos_types::{
    move_types::FunctionId,
    moveos_std::{object::ObjectMeta, tx_context::TxContext},
    transaction::{FunctionCall, VerifiedMoveAction, VerifiedMoveOSTransaction},
};
use serde::de::{self, Visitor};
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

fn consume_any<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    struct AnyVisitor;

    impl<'de> Visitor<'de> for AnyVisitor {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("any value")
        }

        fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(())
        }

        fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(())
        }

        // 实现其他 visit_* 方法...

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            while let Some((_, _)) = map.next_entry::<de::IgnoredAny, de::IgnoredAny>()? {}
            Ok(())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            while let Some(_) = seq.next_element::<de::IgnoredAny>()? {}
            Ok(())
        }
    }

    deserializer.deserialize_any(AnyVisitor)
}

#[derive(Debug, Clone, Serialize)]
pub struct NewVerifiedMoveOSTransaction {
    pub root: ObjectMeta,
    pub ctx: TxContext,
    pub action: VerifiedMoveAction,
}

impl<'de> Deserialize<'de> for NewVerifiedMoveOSTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(unused_variables)]
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct Old {
            root: ObjectMeta,
            ctx: TxContext,
            action: VerifiedMoveAction,
            // #[serde(deserialize_with = "consume_any", default)]
            // skipped_field1: (),
            // #[serde(deserialize_with = "consume_any", default)]
            // skipped_field2: (),
            #[serde(skip_deserializing, skip_serializing)]
            pre_execute_functions: Option<Vec<FunctionCall>>,
            #[serde(skip_deserializing, skip_serializing)]
            post_execute_functions: Option<Vec<FunctionCall>>,
        }
        // let old = Old::deserialize(deserializer)?;
        // Ok(NewVerifiedMoveOSTransaction {
        //     root: old.root,
        //     ctx: old.ctx,
        //     action: old.action,
        // })
        #[derive(Deserialize)]
        enum Compat {
            New(NewVerifiedMoveOSTransaction),
            Old(Old),
        }

        let compat = Compat::deserialize(deserializer)?;
        match compat {
            Compat::New(new) => Ok(new),
            Compat::Old(old) => Ok(NewVerifiedMoveOSTransaction {
                root: old.root,
                ctx: old.ctx,
                action: old.action,
            }),
        }
    }
}

#[test]
fn test() {
    let root = ObjectMeta::genesis_root();
    let ctx = TxContext::random_for_testing_only();
    let action = VerifiedMoveAction::Function {
        call: FunctionCall::new(
            FunctionId::new(
                ModuleId::new(AccountAddress::ONE, Identifier::from_str("test").unwrap()),
                Identifier::from_str("test").unwrap(),
            ),
            vec![],
            vec![],
        ),
        bypass_visibility: false,
    };
    let old = VerifiedMoveOSTransaction::new(root, ctx, action);
    let old_bytes = bcs::to_bytes(&old).unwrap();
    let new: NewVerifiedMoveOSTransaction = bcs::from_bytes(&old_bytes).unwrap();
    let new_bytes = bcs::to_bytes(&new).unwrap();
    let _new2: NewVerifiedMoveOSTransaction = bcs::from_bytes(&new_bytes).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OldStruct {
    value: u64,
}

#[derive(Debug, Clone, Serialize)]
struct NewStruct {
    value: u64,
    #[serde(default)]
    hash: Option<U256>,
}

impl<'de> Deserialize<'de> for NewStruct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NewStructVisitor;

        impl<'de> serde::de::Visitor<'de> for NewStructVisitor {
            type Value = NewStruct;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct NewStruct")
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
                where
                    E: de::Error, {
                println!("visit_u64");
                unimplemented!()
            }

            fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
                where
                    A: de::MapAccess<'de>, {
                println!("visit_map");
                unimplemented!()
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                println!("visit_seq");
                println!("seq_size: {:?}", seq.size_hint());
                let value = match seq
                    .next_element(){
                        Ok(Some(value)) => value,
                        Ok(None) => 0,
                        Err(e) => {
                            println!("error: {:?}", e);
                            0
                        },
                };
                println!("value: {:?}", value);
                let hash = match seq.next_element() {
                    Ok(Some(hash)) => Some(hash),
                    Ok(None) => None,
                    Err(_) => None,
                };
                Ok(NewStruct { value, hash })
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>, {
                println!("visit_newtype_struct");
                deserializer.deserialize_seq(NewStructVisitor)
            }
        }

        deserializer.deserialize_newtype_struct("NewStruct", NewStructVisitor)
    }
}

#[test]
fn test2() {
    let old = OldStruct { value: 1 };
    let old_bytes = bcs::to_bytes(&old).unwrap();
    let mut new: NewStruct = bcs::from_bytes(&old_bytes).unwrap();
    new.hash = Some(U256::from(2u64));
    let new_bytes = bcs::to_bytes(&new).unwrap();
    let _new2: NewStruct = bcs::from_bytes(&new_bytes).unwrap();
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{de, ser};
use std::fmt;
macro_rules! derive_code_try_from_repr {
    (
        #[repr($repr_ty:ident)]
        $( #[$metas:meta] )*
        $vis:vis enum $enum_name:ident {
            $(
                $variant:ident = $value: expr
            ),*
            $( , )?
        }
    ) => {
        #[repr($repr_ty)]
        $( #[$metas] )*
        $vis enum $enum_name {
            $(
                $variant = $value
            ),*
        }

        impl std::convert::TryFrom<$repr_ty> for $enum_name {
            type Error = &'static str;
            fn try_from(value: $repr_ty) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $value => Ok($enum_name::$variant),
                    )*
                    _ => Err("invalid ErrorCode"),
                }
            }
        }

        #[cfg(any(test, feature = "fuzzing"))]
        const ERROR_CODE_VALUES: &'static [$repr_ty] = &[
            $($value),*
        ];
    };
}

derive_code_try_from_repr! {
#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ErrorCode {
    MALFORMED_METADATA = 10000,
    FUNCTION_NOT_EXITS = 10001,
    STRUCT_NOT_EXISTS = 10002,
    NOT_ENOUGH_PARAMETERS = 10003,
    TOO_MANY_PARAMETERS = 10004,
    TYPE_MISMATCH = 10005,
    MALFORMED_STRUCT_NAME = 10006,
    MALFORMED_FUNCTION_NAME = 10007,
    INVALID_DATA_STRUCT = 10008,
    INVALID_DATA_STRUCT_TYPE = 10009,
    INVALID_MODULE_OWNER = 10010,
    INVALID_PRIVATE_GENERICS_TYPE = 10011,
    INVALID_DATA_STRUCT_WITHOUT_DROP_ABILITY = 10012,
    INVALID_DATA_STRUCT_WITHOUT_COPY_ABILITY = 10013,
    INVALID_DATA_STRUCT_NOT_ALLOWED_TYPE = 10014,
    INVALID_DATA_STRUCT_NOT_IN_MODULE_METADATA = 10015,
    INVALID_DATA_STRUCT_WITH_TYPE_PARAMETER = 10016,
    INVALID_DATA_STRUCT_OPTION_WITHOUT_TYPE_PARAMETER = 10017,

    INVALID_ENTRY_FUNC_SIGNATURE = 11000,
    INVALID_PARAM_TYPE_ENTRY_FUNCTION = 11001,

    INVALID_PUBLIC_INIT_FUNC = 12000,
    INVALID_INIT_FUNC_WITH_ENTRY = 12001,
    INVALID_TOO_MANY_PARAMS_INIT_FUNC = 12002,

    INVALID_INSTRUCTION = 13000,

    UNKNOWN_CODE = 18446744073709551615,
}
}

impl ser::Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_u64((*self).into())
    }
}

impl<'de> de::Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ErrorCodeVisitor;
        impl<'de> de::Visitor<'de> for ErrorCodeVisitor {
            type Value = ErrorCode;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("StatusCode as u64")
            }

            fn visit_u64<E>(self, v: u64) -> Result<ErrorCode, E>
            where
                E: de::Error,
            {
                Ok(ErrorCode::try_from(v).unwrap_or(ErrorCode::UNKNOWN_CODE))
            }
        }

        deserializer.deserialize_u64(ErrorCodeVisitor)
    }
}

impl From<ErrorCode> for u64 {
    fn from(status: ErrorCode) -> u64 {
        status as u64
    }
}

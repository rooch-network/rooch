// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use anyhow::{bail, Result};
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::fmt;
use std::str::FromStr;

const MODULE_NAME: &IdentStr = ident_str!("decimal_value");

/// `DecimalValue` is represented `moveos_std::decimal_value::DecimalValue` in Move.
#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub struct DecimalValue {
    pub value: U256,
    pub decimal: u8,
}

impl DecimalValue {
    pub fn new(value: U256, decimal: u8) -> Self {
        Self { value, decimal }
    }

    pub fn from_number(number: Number) -> Result<Self> {
        match number.as_f64() {
            Some(f) => {
                let f_str = f.to_string();
                Ok(Self::from_str(&f_str)?)
            }
            None => {
                match number.as_u64() {
                    Some(u) => {
                        let value = U256::from(u);
                        Ok(Self { value, decimal: 0 })
                    }
                    None => {
                        //Do not support i64
                        bail!("Unsupported number type: {:?}", number);
                    }
                }
            }
        }
    }

    pub fn to_number(&self) -> Result<Number> {
        let value: u128 = self
            .value
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert DecimalValue to Number"))?;

        if self.decimal == 0 {
            Number::from_u128(value).ok_or_else(|| {
                anyhow::anyhow!("Failed to convert DecimalValue to Number: {}", value)
            })
        } else {
            // Check if value is too large for f64
            if value > (1u128 << 53) {
                return Err(anyhow::anyhow!(
                    "Value too large for f64 conversion: {}",
                    value
                ));
            }

            let divisor = 10u64.pow(self.decimal as u32) as f64;
            let f = (value as f64) / divisor;

            Number::from_f64(f)
                .ok_or_else(|| anyhow::anyhow!("Failed to convert DecimalValue to Number"))
        }
    }

    pub fn from_json_value(value: &serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Number(number) => Self::from_number(number.clone()),
            serde_json::Value::String(s) => Self::from_str(s),
            _ => bail!("Unsupported json value: {:?}", value),
        }
    }

    /// Convert `DecimalValue` to `serde_json::Value`.
    /// we use `serde_json::Value::Number` if the decimal value can be represented as a number,
    /// otherwise we use `serde_json::Value::String`.
    pub fn to_json_value(&self) -> serde_json::Value {
        match self.to_number() {
            Ok(number) => serde_json::Value::Number(number),
            Err(_) => serde_json::Value::String(self.to_string()),
        }
    }
}

impl fmt::Display for DecimalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut value_str = self.value.to_string();
        let decimal = self.decimal as usize;

        if decimal == 0 {
            return write!(f, "{}", value_str);
        }

        // Pad with leading zeros if necessary
        while value_str.len() <= decimal {
            value_str.insert(0, '0');
        }

        // Insert decimal point
        let len = value_str.len();
        value_str.insert(len - decimal, '.');

        // Trim trailing zeros after decimal point
        // let trimmed = value_str.trim_end_matches('0');
        let result = value_str.trim_end_matches('.');

        write!(f, "{}", result)
    }
}

impl FromStr for DecimalValue {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(anyhow::anyhow!("Invalid decimal value str: {}", s));
        }

        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => {
                // No decimal point
                let value = U256::from_str(s).map_err(|e| {
                    anyhow::anyhow!(format!("invalid decimal u256 number: {:?}", e))
                })?;
                Ok(DecimalValue { value, decimal: 0 })
            }
            2 => {
                let integer = parts[0];
                let fraction = parts[1];

                let decimal = fraction.len() as u8;
                let mut value_str = integer.to_string();
                value_str.push_str(fraction);

                let value = U256::from_str(&value_str).map_err(|e| {
                    anyhow::anyhow!(format!("invalid decimal u256 number: {:?}", e))
                })?;

                Ok(DecimalValue { value, decimal })
            }
            _ => Err(anyhow::anyhow!("Invalid decimal value format: {}", s)),
        }
    }
}

impl MoveStructType for DecimalValue {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DecimalValue");
}

impl MoveStructState for DecimalValue {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U256,
            move_core_types::value::MoveTypeLayout::U8,
        ])
    }
}

impl PartialEq<DecimalValue> for DecimalValue {
    fn eq(&self, other: &DecimalValue) -> bool {
        if self.decimal == other.decimal {
            // If decimals are the same, we can compare values directly
            return self.value == other.value;
        }

        // Otherwise, we need to normalize the values to compare
        let (adjusted_self, adjusted_other) = if self.decimal > other.decimal {
            // Scale up other's value
            let scale_factor = 10u64.pow((self.decimal - other.decimal) as u32);
            let scaled_other = other.value * U256::from(scale_factor);
            (self.value, scaled_other)
        } else {
            // Scale up self's value
            let scale_factor = 10u64.pow((other.decimal - self.decimal) as u32);
            let scaled_self = self.value * U256::from(scale_factor);
            (scaled_self, other.value)
        };

        adjusted_self == adjusted_other
    }
}

impl TryFrom<AnnotatedMoveStruct> for DecimalValue {
    type Error = anyhow::Error;

    fn try_from(annotated_move_struct: AnnotatedMoveStruct) -> Result<Self, Self::Error> {
        DecimalValue::try_from(&annotated_move_struct)
    }
}

impl TryFrom<&AnnotatedMoveStruct> for DecimalValue {
    type Error = anyhow::Error;

    fn try_from(annotated_move_struct: &AnnotatedMoveStruct) -> Result<Self, Self::Error> {
        let mut fields = annotated_move_struct.value.iter();
        let (value_field_name, value_field_value) = fields
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid DecimalValue"))?;
        let (decimal_field_name, decimal_field_value) = fields
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid DecimalValue"))?;
        debug_assert!(value_field_name.as_str() == "value");
        debug_assert!(decimal_field_name.as_str() == "decimal");

        let value = match value_field_value {
            AnnotatedMoveValue::U256(value) => value,
            _ => return Err(anyhow::anyhow!("Invalid DecimalValue")),
        };
        let decimal = match decimal_field_value {
            AnnotatedMoveValue::U8(decimal) => decimal,
            _ => return Err(anyhow::anyhow!("Invalid DecimalValue")),
        };
        Ok(DecimalValue {
            value: *value,
            decimal: *decimal,
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum NumberOrString {
    Number(Number),
    String(String),
}

impl TryFrom<NumberOrString> for DecimalValue {
    type Error = anyhow::Error;

    fn try_from(value: NumberOrString) -> Result<Self, Self::Error> {
        match value {
            NumberOrString::Number(number) => DecimalValue::from_number(number),
            NumberOrString::String(s) => DecimalValue::from_str(&s),
        }
    }
}

impl From<DecimalValue> for NumberOrString {
    fn from(decimal_value: DecimalValue) -> Self {
        match decimal_value.to_number() {
            Ok(number) => NumberOrString::Number(number),
            Err(_) => NumberOrString::String(decimal_value.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct BCSDecimalValue {
    value: U256,
    decimal: u8,
}

impl From<DecimalValue> for BCSDecimalValue {
    fn from(decimal_value: DecimalValue) -> Self {
        BCSDecimalValue {
            value: decimal_value.value,
            decimal: decimal_value.decimal,
        }
    }
}

impl From<BCSDecimalValue> for DecimalValue {
    fn from(bcs_decimal_value: BCSDecimalValue) -> Self {
        DecimalValue {
            value: bcs_decimal_value.value,
            decimal: bcs_decimal_value.decimal,
        }
    }
}

impl<'de> Deserialize<'de> for DecimalValue {
    fn deserialize<D>(deserializer: D) -> Result<DecimalValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = NumberOrString::deserialize(deserializer)?;
            DecimalValue::try_from(s).map_err(serde::de::Error::custom)
        } else {
            let bcs_decimal_value = BCSDecimalValue::deserialize(deserializer)?;
            Ok(DecimalValue::from(bcs_decimal_value))
        }
    }
}

impl Serialize for DecimalValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let s = NumberOrString::from(self.clone());
            s.serialize(serializer)
        } else {
            let bcs_decimal_value = BCSDecimalValue::from(self.clone());
            bcs_decimal_value.serialize(serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::moveos_std::decimal_value::DecimalValue;
    use move_core_types::u256::U256;

    #[test]
    fn test_decimal_value_tostring() {
        let cases = vec![
            (
                DecimalValue {
                    value: U256::from(1234567u64),
                    decimal: 3,
                },
                "1234.567",
            ),
            (
                DecimalValue {
                    value: U256::from(1000u64),
                    decimal: 3,
                },
                "1.000",
            ),
            (
                DecimalValue {
                    value: U256::from(1u64),
                    decimal: 3,
                },
                "0.001",
            ),
            (
                DecimalValue {
                    value: U256::from(0u64),
                    decimal: 2,
                },
                "0.00",
            ),
            (
                DecimalValue {
                    value: U256::from(1234u64),
                    decimal: 0,
                },
                "1234",
            ),
            (
                DecimalValue {
                    value: U256::from(1000u64),
                    decimal: 4,
                },
                "0.1000",
            ),
        ];

        for (value, expected) in cases {
            assert_eq!(value.to_string(), expected);
        }
    }

    #[test]
    fn test_decimal_value_from_str() {
        // Test successful cases
        let test_cases = vec![
            (
                "1234.567",
                DecimalValue {
                    value: U256::from(1234567u64),
                    decimal: 3,
                },
            ),
            (
                "0.123",
                DecimalValue {
                    value: U256::from(123u64),
                    decimal: 3,
                },
            ),
            (
                "1234",
                DecimalValue {
                    value: U256::from(1234u64),
                    decimal: 0,
                },
            ),
            (
                "0.000123",
                DecimalValue {
                    value: U256::from(123u64),
                    decimal: 6,
                },
            ),
        ];

        for (input, expected) in test_cases {
            let parsed: DecimalValue = input.parse().unwrap();
            assert_eq!(parsed.value, expected.value);
            assert_eq!(parsed.decimal, expected.decimal);
            assert_eq!(parsed.to_string(), input);
        }

        // Test ok cases
        assert!(".123".parse::<DecimalValue>().is_ok());
        assert!("123.".parse::<DecimalValue>().is_ok());

        // Test error cases
        assert!("".parse::<DecimalValue>().is_err());
        assert!("123.456.789".parse::<DecimalValue>().is_err());
        assert!("abc".parse::<DecimalValue>().is_err());
    }

    #[test]
    fn test_decimal_value_from_number() {
        let test_cases = vec![
            serde_json::Number::from_f64(1234.567).unwrap(),
            serde_json::Number::from_f64(0.123).unwrap(),
            serde_json::Number::from_f64(1234.00000).unwrap(),
            serde_json::Number::from_f64(0.000123).unwrap(),
            serde_json::Number::from_u128(1234).unwrap(),
            serde_json::Number::from_u128(1234567).unwrap(),
        ];

        for input in test_cases {
            let parsed = DecimalValue::from_number(input.clone()).unwrap();
            let parsed2 = DecimalValue::from_number(parsed.to_number().unwrap()).unwrap();
            assert_eq!(parsed, parsed2);
        }

        // Test error cases
        assert!(DecimalValue::from_number(serde_json::Number::from_i128(-1234).unwrap()).is_err());
    }

    #[test]
    fn test_decimal_value_large_numbers() {
        let large_value = DecimalValue {
            value: U256::from(u128::MAX),
            decimal: 2,
        };
        assert!(large_value.to_number().is_err());

        let safe_value = DecimalValue {
            value: U256::from(1u128 << 53),
            decimal: 2,
        };
        assert!(safe_value.to_number().is_ok());
    }

    #[test]
    fn test_decimal_value_serialize_deserialize() {
        let decimal_value = DecimalValue {
            value: U256::from(1234567u64),
            decimal: 3,
        };

        let serialized = serde_json::to_string(&decimal_value).unwrap();
        //number
        assert_eq!(serialized, "1234.567");
        //println!("serialized: {}", serialized);
        let deserialized: DecimalValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, decimal_value);

        let decimal_value = DecimalValue {
            value: U256::from(u128::MAX),
            decimal: 3,
        };

        let serialized = serde_json::to_string(&decimal_value).unwrap();
        //println!("serialized: {}", serialized);
        //string
        assert_eq!(serialized, "\"340282366920938463463374607431768211.455\"");
        let deserialized: DecimalValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, decimal_value);
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const MODULE_NAME: &IdentStr = ident_str!("decimal_value");

/// `DecimalValue` is represented `moveos_std::decimal_value::DecimalValue` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DecimalValue {
    pub value: U256,
    pub decimal: u8,
}

impl DecimalValue {
    pub fn new(value: U256, decimal: u8) -> Self {
        Self { value, decimal }
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
}

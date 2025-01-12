// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;

const LOWER_BYTES_UNITS: [&str; 7] = ["b", "k", "m", "g", "t", "p", "e"];

pub fn human_readable_bytes(bytes: u64) -> String {
    let mut v = bytes as f64;
    let mut unit_index = 0;

    while v >= 1024.0 && unit_index < LOWER_BYTES_UNITS.len() - 1 {
        v /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2}{}", v, LOWER_BYTES_UNITS[unit_index])
}

/// Parses a string like "1k", "10M", etc., into the equivalent value in bytes (u64).
pub fn parse_bytes(input: &str) -> anyhow::Result<u64> {
    if input.is_empty() {
        bail!("Input is empty");
    }

    let chars = input.chars();
    let mut value_str = String::new();
    let mut unit = None;

    for c in chars {
        if c.is_ascii_digit() || c == '.' {
            value_str.push(c);
        } else {
            unit = Some(c.to_lowercase().to_string());
            break;
        }
    }

    // Parse the numeric value
    let value: f64 = value_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid number format"))?;

    // Match the unit
    let multiplier = if let Some(unit) = unit {
        match LOWER_BYTES_UNITS.iter().position(|&u| u == unit.as_str()) {
            Some(index) => 1024u64.saturating_pow(index as u32),
            None => bail!("Unrecognized unit: {}", unit),
        }
    } else {
        1u64 // Default to bytes
    };

    // Compute the total value in bytes
    let result = value * multiplier as f64;

    // Ensure it's within the range of u64
    if result > u64::MAX as f64 {
        bail!("Value overflowed u64");
    }

    Ok(result as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_readable_bytes() {
        let test_cases = [
            (0, "0.00b"),
            (1 << 10, "1.00k"),
            (1 << 20, "1.00m"),
            (1 << 30, "1.00g"),
            (1 << 40, "1.00t"),
            (1 << 50, "1.00p"),
            (1 << 60, "1.00e"),
        ];

        for (bytes, expected) in test_cases.iter() {
            assert_eq!(human_readable_bytes(*bytes), *expected);
        }
    }

    #[test]
    fn test_parse_bytes() {
        let test_cases = [
            ("0", 0),
            ("1", 1),
            ("1.5", 1),
            ("1.5K", (1 << 10) * 3 / 2),
            ("1.4K", 1433),
            ("1.5M", (1 << 20) * 3 / 2),
            ("1.5G", (1 << 30) * 3 / 2),
            ("1.5T", (1 << 40) * 3 / 2),
            ("1.5P", (1 << 50) * 3 / 2),
            ("1.5E", (1 << 60) * 3 / 2),
            ("1.5k", 1536),
            ("1.512k", 1548),
        ];

        for (input, expected) in test_cases.iter() {
            assert_eq!(parse_bytes(input).unwrap(), *expected);
        }
    }
}

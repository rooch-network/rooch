// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_readable_bytes() {
        let test_cases = [
            (0, "0.00b"),
            (1024, "1.00k"),
            (1024 * 1024, "1.00m"),
            (1024 * 1024 * 1024, "1.00g"),
            (1024_u64 * 1024 * 1024 * 1024, "1.00t"),
            (1024_u64 * 1024 * 1024 * 1024 * 1024, "1.00p"),
            (1024_u64 * 1024 * 1024 * 1024 * 1024 * 1024, "1.00e"),
        ];

        for (bytes, expected) in test_cases.iter() {
            assert_eq!(human_readable_bytes(*bytes), *expected);
        }
    }
}

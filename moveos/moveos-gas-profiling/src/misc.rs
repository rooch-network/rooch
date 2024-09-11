// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub fn strip_trailing_zeros_and_decimal_point(mut s: &str) -> &str {
    loop {
        match s {
            "0" | ".0" => return s,
            _ => match s.strip_suffix('0') {
                Some(stripped) => s = stripped,
                None => break,
            },
        }
    }
    s.strip_suffix('.').unwrap_or(s)
}

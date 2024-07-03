// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub fn human_readable_size(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    let mut size = bytes as f64;
    let mut unit = units[0];

    for &u in &units[1..] {
        if size < 1024.0 {
            break;
        }
        size /= 1024.0;
        unit = u;
    }

    format!("{:.2} {}", size, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_readable_size() {
        assert_eq!(human_readable_size(0), "0.00 B");
        assert_eq!(human_readable_size(1024), "1.00 KB");
        assert_eq!(human_readable_size(1024 * 1024), "1.00 MB");
        assert_eq!(human_readable_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(human_readable_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
        assert_eq!(
            human_readable_size(1024 * 1024 * 1024 * 1024 * 1024),
            "1.00 PB"
        );
        assert_eq!(
            human_readable_size(1024 * 1024 * 1024 * 1024 * 1024 * 1024),
            "1.00 EB"
        );
    }
}

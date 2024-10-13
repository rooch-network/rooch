// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::blockdata::script::Script;

#[test]
fn test_push_int_cases() {
    let inputs = [
        0i64,
        1i64,
        16i64,
        17i64,
        75i64,
        76i64,
        100i64,
        255i64,
        256i64,
        65535i64,
        65536i64,
        16777215i64,
        16777216i64,
        2147483647i64,
        2147483648i64,
        4294967295i64,
        4294967296i64,
        1000000000000i64,
        -1i64,
        -100i64,
        -1000000000000i64,
    ];

    let expected_outputs = vec![
        vec![0x00],
        vec![0x51],
        vec![0x60],
        vec![0x01, 0x11],
        vec![0x01, 0x4b],
        vec![0x01, 0x4c],
        vec![0x01, 0x64],
        vec![0x02, 0xff, 0x00],
        vec![0x02, 0x00, 0x01],
        vec![0x03, 0xff, 0xff, 0x00],
        vec![0x03, 0x00, 0x00, 0x01],
        vec![0x04, 0xff, 0xff, 0xff, 0x00],
        vec![0x04, 0x00, 0x00, 0x00, 0x01],
        vec![0x04, 0xff, 0xff, 0xff, 0x7f],
        vec![0x05, 0x00, 0x00, 0x00, 0x80, 0x00],
        vec![0x05, 0xff, 0xff, 0xff, 0xff, 0x00],
        vec![0x05, 0x00, 0x00, 0x00, 0x00, 0x01],
        vec![0x06, 0x00, 0x10, 0xa5, 0xd4, 0xe8, 0x00],
        //-1
        vec![0x4f],
        //-100
        vec![0x01, 0xe4],
        //-1000000000000
        vec![0x06, 0x00, 0x10, 0xa5, 0xd4, 0xe8, 0x80],
    ];

    for (i, (input, expected)) in inputs.iter().zip(expected_outputs.iter()).enumerate() {
        let script = Script::builder().push_int(*input).into_script();
        let result = script.as_bytes();

        // println!("input: {}", input);
        // println!("result: {:?}", hex::encode(result));
        // println!("expected: {:?}", hex::encode(expected));

        assert_eq!(result, expected, "Test case {} failed", i);
    }
}

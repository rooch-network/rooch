// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// This file provides a weak symbol for __rust_probestack to fix linking issues
// with wasmer 4.3.x on Rust 1.91.x
//
// See: https://github.com/rust-lang/rust/issues/142612
//      https://github.com/rust-lang/rust/issues/143835

#if defined(__x86_64__)
__attribute__((weak))
void __rust_probestack(void) {
}
#endif

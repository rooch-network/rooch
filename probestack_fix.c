// Workaround for wasmer VM __rust_probestack undefined symbol issue with Rust 1.91
// This provides a weak definition that can be overridden if needed

__attribute__((weak))
void __rust_probestack(void) {
    // Empty implementation - weak symbol
}
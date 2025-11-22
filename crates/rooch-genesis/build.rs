// Build script to link probestack fix for Rust 1.91 + wasmer compatibility

fn main() {
    // Fix for wasmer VM __rust_probestack undefined symbol with Rust 1.91
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");

    // Compile the C file that provides a weak __rust_probestack symbol
    let c_file_path = "../../probestack_fix.c";
    println!("cargo:rerun-if-changed={}", c_file_path);

    #[cfg(target_arch = "x86_64")]
    {
        // Use cc crate to compile the C file
        cc::Build::new().file(c_file_path).compile("probestack_fix");

        println!("cargo:rustc-link-lib=static=probestack_fix");
    }
}

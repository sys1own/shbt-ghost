use std::path::PathBuf;

fn main() {
    let kernel_dir = PathBuf::from("../../kernel");
    println!("cargo:rerun-if-changed={}", kernel_dir.join("shbt_ghost_kernel.c").display());
    println!("cargo:rerun-if-changed={}", kernel_dir.join("include/shbt_hardware.h").display());
    cc::Build::new()
        .file(kernel_dir.join("shbt_ghost_kernel.c"))
        .include(&kernel_dir)
        .include(kernel_dir.join("include"))
        .flag_if_supported("-std=c11")
        .define("SHBT_HOSTED_TEST", None)
        .compile("shbt_ghost_kernel");
}

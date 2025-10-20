use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Step 0: Ensure Sedna sources are available
    ensure_sedna_sources();

    println!("cargo:rerun-if-changed=sednaxml-ref/");

    // Step 1: Build full Sedna server with CMake (this generates necessary headers)
    let sedna_build = build_sedna_server();

    // Step 2: Compile libsedna.c for FFI client (using headers from CMake build)
    compile_libsedna(&sedna_build);

    // Step 3: Copy binaries and share files to project directory for RustEmbed
    copy_binaries_for_embed(&sedna_build);
}

fn ensure_sedna_sources() {
    let sedna_dir = PathBuf::from("sednaxml-ref");

    if !sedna_dir.exists() {
        println!("cargo:warning=Sedna sources not found, cloning from GitHub...");

        let status = Command::new("git")
            .args(&[
                "clone",
                "https://github.com/ParapluOU/sednaxml.git",
                "sednaxml-ref"
            ])
            .status()
            .expect("Failed to execute git clone");

        if !status.success() {
            panic!("Failed to clone Sedna sources from GitHub");
        }

        println!("cargo:warning=Sedna sources cloned successfully");
    }
}

fn build_sedna_server() -> PathBuf {
    // Build Sedna with CMake
    let dst = cmake::Config::new("sednaxml-ref")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("JAVA_DRIVER", "OFF")
        .define("MAKE_DOC", "OFF")
        .define("ENHANCE_TERM", "None")
        .define("ENABLE_TRIGGERS", "ON")
        .define("ENABLE_FTSEARCH", "ON")
        .build();

    dst
}

fn compile_libsedna(sedna_build: &PathBuf) {
    // CMake already built libsedna.a for us, so we just need to link against it
    // The library is at: <sedna_build>/driver/c/libsedna.a
    let driver_dir = sedna_build.join("driver/c");

    // Tell cargo to link against the static library
    println!("cargo:rustc-link-search=native={}", driver_dir.display());
    println!("cargo:rustc-link-lib=static=sedna");

    println!("cargo:rerun-if-changed=sednaxml-ref/driver/c/libsedna.c");
    println!("cargo:rerun-if-changed=sednaxml-ref/driver/c/libsedna.h");
    println!("cargo:rerun-if-changed=sednaxml-ref/driver/c/sp_defs.h");
}

fn copy_binaries_for_embed(install_dir: &PathBuf) {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let embed_dir = manifest_dir.join("sedna_install");

    // Remove old directory if it exists
    let _ = fs::remove_dir_all(&embed_dir);

    // Copy the entire CMake install directory
    copy_dir_recursive(&install_dir, &embed_dir)
        .expect("Failed to copy Sedna install directory");

    println!("cargo:rerun-if-changed=sedna_install/");
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

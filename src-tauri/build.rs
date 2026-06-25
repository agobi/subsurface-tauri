// AI-generated (Claude)
fn main() {
    tauri_build::build();

    // Build libdivecomputer as a static library via CMake.
    // CMakeLists.txt is in src-tauri/libdc/ (created alongside the submodule
    // because the Subsurface-DS9 branch uses autotools, not cmake).
    let dst = cmake::Config::new("libdc")
        .define("LIBDC_WITH_TESTS", "OFF")
        .build();

    // The cmake crate installs artifacts into dst/.
    // `install(TARGETS … ARCHIVE DESTINATION lib)` puts the .a there.
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/src", dst.display());
    println!("cargo:rustc-link-lib=static=divecomputer");

    // Platform system libs required by libdivecomputer.
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=udev");
        println!("cargo:rustc-link-lib=usb-1.0");
    }
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
    }
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=setupapi");
        println!("cargo:rustc-link-lib=hid");
    }

    // Generate FFI bindings.
    // After cmake runs, version.h is installed to dst/include/libdivecomputer/.
    // We pass that directory first so bindgen can resolve the generated header.
    let dst_include = format!("{}/include", dst.display());
    let bindings = bindgen::Builder::default()
        .header("libdc/include/libdivecomputer.h")
        .clang_arg(format!("-I{}", dst_include))
        .clang_arg("-Ilibdc/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("dc_.*")
        .allowlist_function("dc_.*")
        .allowlist_var("DC_.*")
        .generate()
        .expect("failed to generate libdivecomputer bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ffi.rs"))
        .expect("failed to write ffi.rs");
}

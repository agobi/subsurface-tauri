// AI-generated (Claude)
fn main() {
    tauri_build::build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // SPIKE (2026-07-01): temporarily attempt the cross-compile for every
    // target, including Android, to find out what breaks. See
    // docs/superpowers/specs/2026-07-01-android-libdc-build-spike-design.md.
    {
        // Build libdivecomputer as a static library via CMake.
        // CMakeLists.txt is in src-tauri/libdc-cmake/ (tracked in the parent repo).
        // The actual C sources live in the sibling libdc/ git submodule.
        let mut cmake_config = cmake::Config::new("libdc-cmake");
        cmake_config.define("LIBDC_WITH_TESTS", "OFF");

        if target_os == "android" {
            // The `cmake` crate only drives CMake's own (fragile) built-in
            // Android platform detection unless it sees `ANDROID_ABI` +
            // `CMAKE_TOOLCHAIN_FILE=.../android.toolchain.cmake` — that
            // combination switches it to defer entirely to the NDK's own
            // toolchain file, which is what actually knows how to find the
            // NDK's sysroot/compilers/make program.
            let ndk_home =
                std::env::var("ANDROID_NDK_HOME").expect("ANDROID_NDK_HOME not set");
            cmake_config
                .define(
                    "CMAKE_TOOLCHAIN_FILE",
                    format!("{ndk_home}/build/cmake/android.toolchain.cmake"),
                )
                .define("ANDROID_ABI", "arm64-v8a")
                .define("ANDROID_PLATFORM", "android-24"); // matches gen/android/app/build.gradle.kts minSdk
        }

        let dst = cmake_config.build();

        // The cmake crate installs artifacts into dst/.
        // `install(TARGETS … ARCHIVE DESTINATION lib)` puts the .a there.
        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-search=native={}/src", dst.display());
        println!("cargo:rustc-link-lib=static=divecomputer");

        // Platform system libs required by libdivecomputer.
        if target_os == "linux" {
            println!("cargo:rustc-link-lib=udev");
            println!("cargo:rustc-link-lib=usb-1.0");
        } else if target_os == "macos" {
            println!("cargo:rustc-link-lib=framework=IOKit");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
        } else if target_os == "windows" {
            println!("cargo:rustc-link-lib=setupapi");
            println!("cargo:rustc-link-lib=hid");
        }

        // Generate FFI bindings.
        // After cmake runs, version.h is installed to dst/include/libdivecomputer/.
        // We pass that directory first so bindgen can resolve the generated header.
        let dst_include = format!("{}/include", dst.display());
        let bindings = bindgen::Builder::default()
            .header("libdc-cmake/include/libdivecomputer.h")
            .clang_arg(format!("-I{}", dst_include))
            .clang_arg("-Ilibdc/include")
            .clang_arg("-Ilibdc-cmake/include")
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
}

//! Build script for `ass-renderer`.
//!
//! Its only job is to locate and link the native libass library when the
//! dev-only `libass-compare` feature is enabled, so the A/B comparison harness
//! can call libass directly. Discovery is platform-split for consistency:
//! `vcpkg` on Windows (MSVC ABI) and `pkg-config` elsewhere. When the feature is
//! off (the default for every CI lint/test combo) this script does nothing.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_LIBASS_COMPARE").is_none() {
        return;
    }
    link_libass();
}

#[cfg(target_os = "windows")]
fn link_libass() {
    // Honors VCPKG_ROOT; expects `vcpkg install libass:x64-windows-static-md`.
    // The vcpkg crate emits link directives for libass and its transitive deps
    // (freetype, harfbuzz, fribidi, ...).
    match vcpkg::Config::new().find_package("libass") {
        Ok(_) => {}
        Err(e) => panic!(
            "libass-compare: vcpkg could not find libass ({e}).\n\
             Install it with `vcpkg install libass:x64-windows-static-md` and set \
             VCPKG_ROOT, or build without the libass-compare feature."
        ),
    }
    // libass's DirectWrite/GDI font provider (ass_directwrite.c) pulls in Windows
    // system libraries that the vcpkg metadata does not list. Link them here so
    // the static lib resolves.
    for lib in ["gdi32", "user32", "dwrite", "ole32", "oleaut32"] {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }
}

#[cfg(not(target_os = "windows"))]
fn link_libass() {
    // Linux: `apt-get install libass-dev`; macOS: `brew install libass`; or a
    // pinned vcpkg/manifest install with PKG_CONFIG_PATH pointed at it.
    match pkg_config::Config::new().probe("libass") {
        Ok(_) => {}
        Err(e) => panic!(
            "libass-compare: pkg-config could not find libass ({e}).\n\
             Install libass-dev (Linux) / libass (brew) or set PKG_CONFIG_PATH, \
             or build without the libass-compare feature."
        ),
    }
}

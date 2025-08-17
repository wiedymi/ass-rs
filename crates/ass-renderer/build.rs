fn main() {
    // Only link libass when the libass-compare feature is enabled
    #[cfg(feature = "libass-compare")]
    {
        // Use pkg-config to find libass
        if let Err(e) = pkg_config::Config::new()
            .atleast_version("0.14.0")
            .probe("libass")
        {
            // Fallback to manual linking for macOS with Homebrew
            #[cfg(target_os = "macos")]
            {
                println!("cargo:warning=pkg-config failed: {e}, trying homebrew paths");
                println!("cargo:rustc-link-search=/opt/homebrew/lib");
                println!("cargo:rustc-link-search=/usr/local/lib");
                println!("cargo:rustc-link-lib=ass");
            }

            #[cfg(not(target_os = "macos"))]
            {
                panic!("Cannot find libass. Please install libass development files. Error: {e}");
            }
        }
    }
}

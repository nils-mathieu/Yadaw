fn main() {
    if cfg!(all(target_os = "macos", feature = "coreaudio")) {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
    }
}

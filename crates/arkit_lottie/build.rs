fn main() {
    // `thorvg-sys` intentionally suppresses automatic C++ runtime linkage for
    // non-hosted SDK targets. Retain the shared C++ runtime dependency on the
    // final cdylib; the HAP packager is responsible for bundling the matching
    // libc++_shared.so from the OHOS SDK.
    println!("cargo:rustc-link-lib=dylib=c++");
}

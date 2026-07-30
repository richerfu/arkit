//! Locate / build libghostty-vt and emit link flags + bindgen bindings.
//!
//! Resolution order:
//! 1. `GHOSTTY_VT_LIB_DIR` — prebuilt directory containing `libghostty-vt.*`
//! 2. Cached build under `OUT_DIR/ghostty-vt/`
//! 3. Ghostty sources + Zig:
//!    - `GHOSTTY_SRC` if set
//!    - else `vendor/ghostty` (git submodule / published crate contents)
//! 4. feature `stub` / missing native lib on non-OHOS hosts — stub for checks
//!
//! Published crates ship `vendor/ghostty` (see `package.include`). Consumers
//! need a Zig toolchain on PATH (or `ZIG`) to compile libghostty-vt from that
//! tree. Prebuilt overrides remain available via `GHOSTTY_VT_LIB_DIR`.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=scripts/build-ghostty-vt.sh");
    println!("cargo:rerun-if-changed=scripts/fetch-ghostty-deps.sh");
    println!("cargo:rerun-if-changed=vendor/ghostty-include/ghostty/vt.h");
    println!("cargo:rerun-if-changed=vendor/ghostty/build.zig");
    println!("cargo:rerun-if-env-changed=GHOSTTY_SRC");
    println!("cargo:rerun-if-env-changed=GHOSTTY_VT_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ZIG");
    println!("cargo:rerun-if-env-changed=ZIG_TARGET");
    println!("cargo:rerun-if-env-changed=ZIG_GLOBAL_CACHE_DIR");
    println!("cargo:rustc-check-cfg=cfg(ghostty_vt_stub)");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header = manifest_dir.join("vendor/ghostty-include/ghostty/vt.h");
    let include_dir = manifest_dir.join("vendor/ghostty-include");

    if cfg!(feature = "stub") {
        println!("cargo:rustc-cfg=ghostty_vt_stub");
        write_stub_marker(&out_dir);
        return;
    }

    let lib_dir = resolve_lib_dir(&manifest_dir, &out_dir);
    match lib_dir {
        Some(dir) => {
            println!("cargo:rustc-link-search=native={}", dir.display());
            let prefer_static = env::var("GHOSTTY_VT_DYNAMIC").ok().as_deref() != Some("1");
            let has_static =
                dir.join("libghostty-vt.a").exists() || dir.join("ghostty-vt-static.lib").exists();
            let use_static = prefer_static && has_static;
            if use_static {
                // Force the archive (macOS ld otherwise prefers adjacent dylibs).
                println!("cargo:rustc-link-lib=static:+whole-archive=ghostty-vt");
            } else {
                println!("cargo:rustc-link-lib=dylib=ghostty-vt");
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
            }

            // Zig static archives may pull in libc / libm on ELF targets.
            match env::var("CARGO_CFG_TARGET_OS").ok().as_deref() {
                Some("linux") => {
                    println!("cargo:rustc-link-lib=dylib=c");
                    println!("cargo:rustc-link-lib=dylib=m");
                    println!("cargo:rustc-link-lib=dylib=pthread");
                    println!("cargo:rustc-link-lib=dylib=dl");
                }
                Some("macos") => {
                    println!("cargo:rustc-link-lib=dylib=c++");
                }
                _ => {}
            }

            generate_bindings(
                &header,
                &include_dir,
                &out_dir,
                /*static_lib=*/ use_static,
            );
        }
        None => {
            if env::var("CARGO_CFG_TARGET_ENV").ok().as_deref() == Some("ohos") {
                panic!(
                    "libghostty-vt is required for an OHOS build; initialize \
                     vendor/ghostty, set GHOSTTY_SRC / GHOSTTY_VT_LIB_DIR, or \
                     explicitly enable the `stub` feature for a non-production check"
                );
            }
            println!(
                "cargo:warning=libghostty-vt not found on this host; compiling arkit_terminal with stub \
                 (init submodule vendor/ghostty, set GHOSTTY_SRC / GHOSTTY_VT_LIB_DIR, and ensure Zig is on PATH)"
            );
            println!("cargo:rustc-cfg=ghostty_vt_stub");
            write_stub_marker(&out_dir);
        }
    }
}

fn write_stub_marker(out_dir: &Path) {
    let stub_rs = out_dir.join("ghostty_vt_bindings.rs");
    std::fs::write(
        &stub_rs,
        "/* stub mode — bindings not used; see src/ffi.rs */\n",
    )
    .expect("write stub marker");
}

fn resolve_lib_dir(manifest_dir: &Path, out_dir: &Path) -> Option<PathBuf> {
    let arch_key = cargo_arch_key();

    if let Ok(dir) = env::var("GHOSTTY_VT_LIB_DIR") {
        let p = PathBuf::from(&dir);
        // Prefer multi-arch layout: $GHOSTTY_VT_LIB_DIR/{aarch64,armv7,x86_64}/lib
        // so one env var works for ohrs multi-arch builds.
        if let Some(found) = resolve_arch_lib_dir(&p, &arch_key) {
            return Some(found);
        }
        println!(
            "cargo:warning=GHOSTTY_VT_LIB_DIR does not contain libghostty-vt for {arch_key}: {}",
            p.display()
        );
    }

    // Workspace-level multi-arch cache used by CI / local verification.
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(manifest_dir);
    let multi = workspace_root.join("target/ghostty-vt-ohos");
    if let Some(found) = resolve_arch_lib_dir(&multi, &arch_key) {
        return Some(found);
    }

    let candidate = out_dir.join("ghostty-vt");
    if find_lib(&candidate).is_some() {
        return Some(candidate);
    }
    let install_lib = out_dir.join("ghostty-vt/lib");
    if find_lib(&install_lib).is_some() {
        return Some(install_lib);
    }

    let ghostty_src = resolve_ghostty_src(manifest_dir)?;
    let zig = env::var("ZIG")
        .ok()
        .map(PathBuf::from)
        .or_else(|| which("zig"))?;

    let script = manifest_dir.join("scripts/build-ghostty-vt.sh");
    // Per-target install under OUT_DIR so concurrent multi-arch builds don't clash.
    let prefix = out_dir.join(format!("ghostty-vt-{arch_key}"));
    let status = Command::new("bash")
        .arg(&script)
        .env("GHOSTTY_SRC", &ghostty_src)
        .env("ZIG", &zig)
        .env("OUT_DIR", &prefix)
        .env(
            "ZIG_TARGET",
            env::var("ZIG_TARGET").unwrap_or_else(|_| zig_target_from_cargo()),
        )
        .status()
        .ok()?;
    if !status.success() {
        println!(
            "cargo:warning=build-ghostty-vt.sh failed for {} ({arch_key})",
            ghostty_src.display()
        );
        return None;
    }

    if find_lib(&prefix.join("lib")).is_some() {
        Some(prefix.join("lib"))
    } else if find_lib(&prefix).is_some() {
        Some(prefix)
    } else {
        let zig_out = ghostty_src.join("zig-out/lib");
        if find_lib(&zig_out).is_some() {
            Some(zig_out)
        } else {
            None
        }
    }
}

/// Map cargo `CARGO_CFG_TARGET_ARCH` → directory names we use for multi-arch
/// prebuilts (`target/ghostty-vt-ohos/<key>/lib`).
fn cargo_arch_key() -> String {
    match env::var("CARGO_CFG_TARGET_ARCH").ok().as_deref() {
        Some("aarch64") => "aarch64".into(),
        Some("arm") => "armv7".into(),
        Some("x86_64") => "x86_64".into(),
        Some("loongarch64") => "loongarch64".into(),
        Some(other) => other.into(),
        None => "aarch64".into(),
    }
}

/// Resolve `$root/<arch>/lib` or `$root/lib` if a libghostty-vt artifact exists.
fn resolve_arch_lib_dir(root: &Path, arch_key: &str) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }
    if let Some(candidate) = [root.join(arch_key).join("lib"), root.join(arch_key)]
        .into_iter()
        .find(|candidate| find_lib(candidate).is_some())
    {
        return Some(candidate);
    }

    // A multi-arch root must never fall back to a flat artifact from another
    // architecture. Flat layouts remain supported for true single-arch roots.
    let has_arch_layout = ["aarch64", "armv7", "x86_64", "loongarch64"]
        .iter()
        .any(|arch| root.join(arch).is_dir());
    if has_arch_layout {
        return None;
    }

    [root.join("lib"), root.to_path_buf()]
        .into_iter()
        .find(|candidate| find_lib(candidate).is_some())
}

/// Ghostty source tree used to build libghostty-vt.
///
/// Priority:
/// 1. `GHOSTTY_SRC`
/// 2. `vendor/ghostty` next to this crate (submodule / packaged source)
fn resolve_ghostty_src(manifest_dir: &Path) -> Option<PathBuf> {
    if let Ok(src) = env::var("GHOSTTY_SRC") {
        let p = PathBuf::from(src);
        if is_ghostty_tree(&p) {
            return Some(p);
        }
        println!(
            "cargo:warning=GHOSTTY_SRC is set but is not a Ghostty tree: {}",
            p.display()
        );
    }

    let vendored = manifest_dir.join("vendor/ghostty");
    if is_ghostty_tree(&vendored) {
        return Some(vendored);
    }

    println!(
        "cargo:warning=Ghostty sources missing at {} \
         (run: git submodule update --init crates/arkit_terminal/vendor/ghostty)",
        vendored.display()
    );
    None
}

fn is_ghostty_tree(path: &Path) -> bool {
    path.is_dir() && path.join("build.zig").is_file()
}

fn find_lib(dir: &Path) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    for name in [
        "libghostty-vt.a",
        "libghostty-vt.so",
        "libghostty-vt.dylib",
        "ghostty-vt.lib",
        "ghostty-vt-static.lib",
    ] {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(rd) = std::fs::read_dir(dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_file() {
                let n = p.file_name()?.to_str()?;
                if n.contains("ghostty-vt") {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn zig_target_from_cargo() -> String {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "aarch64".into());
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "linux".into());
    let env_abi = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os == "linux" && env_abi == "ohos" {
        return match arch.as_str() {
            "arm" => "arm-linux-ohoseabi".into(),
            _ => format!("{arch}-linux-ohos"),
        };
    }
    match (arch.as_str(), os.as_str(), env_abi.as_str()) {
        (a, "linux", "gnu") => format!("{a}-linux-gnu"),
        (a, "linux", "musl") => format!("{a}-linux-musl"),
        (a, "macos", _) => format!("{a}-macos"),
        (a, "linux", _) => format!("{a}-linux-gnu"),
        _ => String::new(),
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let p = dir.join(bin);
            p.is_file().then_some(p)
        })
    })
}

fn generate_bindings(header: &Path, include_dir: &Path, out_dir: &Path, static_lib: bool) {
    if !header.exists() {
        panic!(
            "missing Ghostty header at {} — vendor/ghostty-include is required",
            header.display()
        );
    }

    let mut builder = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg("-D_GNU_SOURCE")
        .clang_arg("-fparse-all-comments")
        // Terminal core, render-state (primary paint path), formatters, key/mouse
        // encoders for interaction, style/color helpers.
        .allowlist_function("ghostty_terminal_.*")
        .allowlist_function("ghostty_render_.*")
        .allowlist_function("ghostty_formatter_.*")
        .allowlist_function("ghostty_style_.*")
        .allowlist_function("ghostty_key_.*")
        .allowlist_function("ghostty_mouse_.*")
        .allowlist_function("ghostty_focus_.*")
        .allowlist_function("ghostty_cell_.*")
        .allowlist_function("ghostty_unicode_.*")
        .allowlist_function("ghostty_free")
        .allowlist_function("ghostty_alloc")
        .allowlist_type("Ghostty.*")
        .allowlist_type("GHOSTTY_.*")
        .allowlist_var("GHOSTTY_.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if static_lib {
        builder = builder.clang_arg("-DGHOSTTY_STATIC");
    }

    let bindings = builder.generate().expect("bindgen ghostty/vt.h");
    bindings
        .write_to_file(out_dir.join("ghostty_vt_bindings.rs"))
        .expect("write bindings");
}

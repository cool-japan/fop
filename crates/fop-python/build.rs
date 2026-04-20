fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let is_extension = std::env::var("CARGO_FEATURE_EXTENSION_MODULE").is_ok();

    match target_os.as_str() {
        "macos" => {
            // pyo3 0.22+ no longer auto-adds -undefined dynamic_lookup on macOS.
            // maturin handles it; for raw cargo builds we emit it here (cdylib only).
            if is_extension {
                println!("cargo:rustc-cdylib-link-arg=-undefined");
                println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
            }
            // Always link Python on macOS so test binaries resolve pyo3 symbols.
            link_python();
        }
        "linux" => {
            // Extension modules (.so) get Python symbols from the embedding Python
            // process at load time — no explicit link needed.
            // Test binaries and rlib builds must link Python explicitly because
            // pyo3's own build script may not emit the search path on non-standard
            // Python installations (e.g. conda, CUDA containers).
            if !is_extension {
                link_python();
            }
        }
        _ => {}
    }
}

fn link_python() {
    let python = std::env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_string());

    let script = "import sysconfig, sys; \
        d = sysconfig.get_config_var('LIBDIR') or ''; \
        v = sysconfig.get_config_var('LDVERSION') \
            or '{}.{}'.format(*sys.version_info[:2]); \
        fw = sysconfig.get_config_var('PYTHONFRAMEWORKPREFIX') or ''; \
        print(d + '|' + v + '|' + fw)";

    let Ok(out) = std::process::Command::new(&python)
        .args(["-c", script])
        .output()
    else {
        return;
    };

    let text = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<&str> = text.trim().splitn(3, '|').collect();
    if parts.len() != 3 {
        return;
    }

    let (libdir, ldver, fw_prefix) = (parts[0], parts[1], parts[2]);

    if !libdir.is_empty() {
        println!("cargo:rustc-link-search=native={libdir}");
    }
    if !fw_prefix.is_empty() {
        let fw_lib = format!("{fw_prefix}/Frameworks/Python.framework/Versions/{ldver}/lib");
        println!("cargo:rustc-link-search=native={fw_lib}");
        println!("cargo:rustc-link-search=framework={fw_prefix}/Frameworks");
    }
    if !ldver.is_empty() {
        println!("cargo:rustc-link-lib=python{ldver}");
    }
}

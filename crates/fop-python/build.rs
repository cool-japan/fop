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
            link_python();
        }
        "linux" => {
            // Always link Python so workspace test runs resolve pyo3 symbols.
            // maturin handles manylinux production builds separately and does not
            // use this build.rs path, so linking Python here is safe.
            link_python();
        }
        _ => {}
    }
}

/// Emit Cargo link directives to resolve pyo3 Python symbols.
fn link_python() {
    let python = std::env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_string());

    let script = concat!(
        "import sysconfig, sys;",
        "d=sysconfig.get_config_var('LIBDIR') or '';",
        "v=sysconfig.get_config_var('LDVERSION')",
        " or '{}.{}'.format(*sys.version_info[:2]);",
        "fw=sysconfig.get_config_var('PYTHONFRAMEWORKPREFIX') or '';",
        "print(d+'|'+v+'|'+fw)"
    );

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

    // Search paths apply to all link steps — always safe to emit.
    if !libdir.is_empty() {
        println!("cargo:rustc-link-search=native={libdir}");
    }
    if !fw_prefix.is_empty() {
        let fw_lib = format!("{fw_prefix}/Frameworks/Python.framework/Versions/{ldver}/lib");
        println!("cargo:rustc-link-search=native={fw_lib}");
        println!("cargo:rustc-link-search=framework={fw_prefix}/Frameworks");
    }

    // Supplement with common Linux Python library paths.  Environments that use
    // -nodefaultlibs (e.g. CUDA toolchains) won't search /usr/lib automatically.
    for path in [
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
        "/usr/lib",
        "/usr/local/lib",
        "/opt/conda/lib",
        "/usr/lib64",
    ] {
        if std::path::Path::new(path).exists() {
            println!("cargo:rustc-link-search=native={path}");
        }
    }

    if ldver.is_empty() {
        return;
    }

    println!("cargo:rustc-link-lib=python{ldver}");
}

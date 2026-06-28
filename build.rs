fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("FileDescription", "Fivetris - simple Tetris trainer");
        res.set("ProductName", "Fivetris");
        res.set("CompanyName", "Relative");
        res.set("LegalCopyright", "Open-source developer, Relative");
        res.set("OriginalFilename", "fivetris.exe");
        res.compile().expect("failed to compile Windows resources");
    }

    #[cfg(unix)]
    if std::env::var("PROFILE").unwrap() == "release" {
        copy_release_assets();
    }
}

#[cfg(unix)]
fn copy_release_assets() {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::fs;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let release_dir = out_dir.ancestors().nth(3).unwrap().to_path_buf();

    let assets: [(&str, &str); 3] = [
        ("scripts/fivetris.desktop", "fivetris.desktop"),
        ("scripts/install.sh", "install.sh"),
        ("assets/icon.png", "icon.png"),
    ];

    for (src, dst) in &assets {
        let dst_path = release_dir.join(dst);
        fs::copy(manifest_dir.join(src), &dst_path)
            .expect(&format!("failed to copy {} to {:?}", src, dst_path));
    }

    fs::set_permissions(release_dir.join("install.sh"), fs::Permissions::from_mode(0o755))
        .expect("failed to chmod install.sh");

    println!("cargo:rerun-if-changed=scripts/fivetris.desktop");
    println!("cargo:rerun-if-changed=scripts/install.sh");
    println!("cargo:rerun-if-changed=assets/icon.png");
}

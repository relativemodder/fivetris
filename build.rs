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

    #[cfg(target_os = "linux")]
    if std::env::var("PROFILE").unwrap() == "release" {
        copy_linux_assets();
    }

    #[cfg(target_os = "macos")]
    if std::env::var("PROFILE").unwrap() == "release" {
        create_macos_app_bundle();
    }
}

#[cfg(target_os = "linux")]
fn copy_linux_assets() {
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

#[cfg(target_os = "macos")]
fn create_macos_app_bundle() {
    use std::path::PathBuf;
    use std::fs;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let release_dir = out_dir.ancestors().nth(3).unwrap().to_path_buf();

    let app_dir = release_dir.join("fivetris.app");
    let contents_dir = app_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");

    fs::create_dir_all(&macos_dir).expect("failed to create MacOS directory");
    fs::create_dir_all(&resources_dir).expect("failed to create Resources directory");

    let binary_src = release_dir.join("fivetris");
    let binary_dst = macos_dir.join("fivetris");
    if binary_src.exists() {
        fs::copy(&binary_src, &binary_dst)
            .expect("failed to copy binary to .app bundle");
    }

    let icon_src = manifest_dir.join("assets/icon.png");
    let icon_dst = resources_dir.join("icon.png");
    if icon_src.exists() {
        fs::copy(&icon_src, &icon_dst)
            .expect("failed to copy icon to .app bundle");
    }

    let info_plist = contents_dir.join("Info.plist");
    fs::write(&info_plist, r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>fivetris</string>
    <key>CFBundleIdentifier</key>
    <string>io.github.relativemodder.fivetris</string>
    <key>CFBundleName</key>
    <string>Fivetris</string>
    <key>CFBundleDisplayName</key>
    <string>Fivetris</string>
    <key>CFBundleVersion</key>
    <string>0.3.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.3.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>icon.png</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
</dict>
</plist>
"#).expect("failed to write Info.plist");

    println!("cargo:rerun-if-changed=assets/icon.png");
}

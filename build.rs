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
}

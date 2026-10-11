//! Embeds the icon and version information Windows shows for wren.exe.

fn main() {
    println!("cargo:rerun-if-changed=packaging/windows/wren.ico");
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("packaging/windows/wren.ico")
            .set("ProductName", "Wren")
            .set("FileDescription", "Wren")
            .set(
                "LegalCopyright",
                "© 2026 Andrei Antonescu and Wren contributors. MIT licence.",
            );
        if let Err(error) = resource.compile() {
            println!("cargo:warning=could not embed the Windows icon: {error}");
        }
    }
}

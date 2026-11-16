//! Embed the Windows executable icon and version resources.
//!
//! The `.ico` is generated from `assets/logo.svg` by `tools/generate-icons.py`
//! and committed, so this only runs the resource compiler.

fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");

    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/app.ico");
        resource.set("ProductName", "Auto Update Cloud Firewall");
        resource.set(
            "FileDescription",
            "Cloud security-group firewall auto-updater",
        );
        resource.set("LegalCopyright", "Copyright (c) 2026 rustsilk.com");

        // A missing resource compiler should not stop the build; the window and
        // tray icons are embedded independently of this step.
        if let Err(err) = resource.compile() {
            println!("cargo:warning=failed to embed the executable icon: {err}");
        }
    }
}

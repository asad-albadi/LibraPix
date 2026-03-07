fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../../assets/logo/blue/icon.ico");
        resource.set("ProductName", "LibraPix");
        resource.set("FileDescription", "LibraPix desktop media manager");
        resource.set("InternalName", "LibraPix");
        resource.set("OriginalFilename", "librapix-app.exe");
        resource
            .compile()
            .expect("failed to compile Windows resources for librapix-app");
    }
}

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("readme/Logo.ico");
        res.compile().unwrap();
        // unoptimized view! bodies need more than the default 1MB main thread stack
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            println!("cargo:rustc-link-arg-bins=/STACK:8388608");
        }
    }
}

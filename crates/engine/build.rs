fn main() {
    #[cfg(target_env = "msvc")]
    {
        println!("cargo:rustc-link-arg=/STACK:8388608");
    }
}

fn main() {
    cc::Build::new()
        .file("src/xorg.c")
        .compile("moemenu");
    pkg_config::Config::new()
        .atleast_version("1.4.99.1")
        .probe("x11")
        .unwrap();
}

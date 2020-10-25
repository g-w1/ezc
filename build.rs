fn main() {
    std::env::set_current_dir(&std::path::Path::new("./lib")).unwrap();
    std::process::Command::new("zig")
        .arg("build")
        .spawn()
        .expect("could not run zig build, please install zig");
}

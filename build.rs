fn main() {
    if option_env!("HAS_NO_ZIG").is_none() {
        std::env::set_current_dir(&std::path::Path::new("./lib")).unwrap();
        let output = std::process::Command::new("zig")
            .arg("build")
            .output()
            .expect("could not run zig build, please install zig");

        if !output.status.success() {
            eprintln!("{}", std::str::from_utf8(&output.stderr).unwrap());
            eprintln!("{}", std::str::from_utf8(&output.stdout).unwrap());
            panic!("Zig build failed");
        }
    }
}

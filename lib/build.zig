const std = @import("std");
const Builder = std.build.Builder;
const builtin = std.builtin;

pub fn build(b: *Builder) void {
    // const mode = b.standardReleaseOptions();
    const mode = builtin.Mode.ReleaseSmall;
    const lib = b.addStaticLibrary("std", "src/lib.zig");
    lib.bundle_compiler_rt = true;
    lib.setBuildMode(mode);
    lib.install();

    var main_tests = b.addTest("src/lib.zig");
    main_tests.setBuildMode(mode);

    const test_step = b.step("test", "Run library tests");
    test_step.dependOn(&main_tests.step);
}

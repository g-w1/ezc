const std = @import("std");

export fn PutChar(c: u8) i64 {
    const stdout = std.io.getStdOut().outStream();
    // TODO add unicode with {u} when https://github.com/ziglang/zig/pull/6390 is merged
    stdout.print("{c}", .{c}) catch return -1;
    return 0;
}
export fn PutNum(n: i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{}", .{n}) catch return -1;
    return 0;
}

export fn PutNumHex(n: i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{X}", .{n}) catch return -1;
    return 0;
}

export fn PutNumOct(n: i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{X}", .{n}) catch return -1;
    return 0;
}

export fn PutNumBin(n: i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{b}", .{n}) catch return -1;
    return 0;
}

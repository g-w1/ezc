const std = @import("std");

export fn PutString(s: [*]i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    const len: i64 = s[0];
    var were_on_rn: u32 = 1;
    while (were_on_rn < len) : (were_on_rn += 1) {
        stdout.print("{c}", .{@intCast(u8, s[were_on_rn])}) catch return -1;
    }
    return 0;
}

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

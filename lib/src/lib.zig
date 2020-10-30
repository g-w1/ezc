const std = @import("std");

export fn DbgPutString(s: [*]i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    const len: i64 = s[1];
    var were_on_rn: u32 = 2;
    var char: u8 = 0;
    while (were_on_rn < len + 2) : (were_on_rn += 1) { // we do len + 2 because the offset in the beg of array is 2
        char = @intCast(u8, s[were_on_rn]);
        // stdout.print("{}: ", .{char}) catch return -1;
        // stdout.print("{c}\n", .{char}) catch return -1;
        stdout.print("{c}", .{char}) catch return -1;
    }
    return 0;
}
export fn PutString(s: [*]i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    const len: i64 = s[1];
    var were_on_rn: u32 = 2;
    var char: u8 = 0;
    while (were_on_rn < len + 2) : (were_on_rn += 1) { // we do len + 2 because the offset in the beg of array is 2
        char = @intCast(u8, s[were_on_rn]);
        stdout.print("{c}", .{char}) catch return -1;
    }
    return 0;
}
export fn PutStringLine(s: [*]i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    _ = PutString(s);
    stdout.print("\n", .{}) catch return -1;
    return 0;
}

export fn PutNewLine() i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("\n", .{}) catch return -1;
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
    stdout.print("0x{X}", .{n}) catch return -1;
    return 0;
}

export fn PutNumBin(n: i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{b}", .{n}) catch return -1;
    return 0;
}

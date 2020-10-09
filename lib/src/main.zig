const std = @import("std");

export fn PutChar(c: u8) i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{c}", .{c}) catch return -1;
    return 0;
}
export fn PutNum(n: i64) i64 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{}", .{n}) catch return -1;
    return 0;
}

const std = @import("std");

/// print a number to the terminal
export fn PutNum(a: i64) i8 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{}", .{a}) catch return -1;
    return 0;
}

/// print a number to the terminal, but as a character
export fn PutChar(a: u8) i8 {
    const stdout = std.io.getStdOut().outStream();
    stdout.print("{c}", .{a}) catch return -1;
    return 0;
}

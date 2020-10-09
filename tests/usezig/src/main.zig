const std = @import("std");
export fn PutNum(a: i64) i64 {
    std.debug.print("{}", .{a});
    return 0;
}
export fn PutChar(a: u8) u8 {
    std.debug.print("{}", .{@ptrCast(u8, a)});
    return 0;
}

const std = @import("std");

// We define these so we dont have to link against compiler-rt
export fn memcpy(noalias dest: ?[*]u8, noalias src: ?[*]const u8, n: usize) callconv(.C) ?[*]u8 {
    @setRuntimeSafety(false);

    var index: usize = 0;
    while (index != n) : (index += 1)
        dest.?[index] = src.?[index];

    return dest;
}

export fn memset(dest: ?[*]u8, c: u8, n: usize) callconv(.C) ?[*]u8 {
    @setRuntimeSafety(false);

    var index: usize = 0;
    while (index != n) : (index += 1)
        dest.?[index] = c;

    return dest;
}

// The actual stuff

export fn PutString(s: [*]i64) i64 {
    const stdout = std.io.getStdOut().writer();
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
    if (PutString(s) != 0) return -1;
    if (PutNewLine() != 0) return -1;
    return 0;
}

export fn PutNewLine() i64 {
    const stdout = std.io.getStdOut().writer();
    stdout.print("\n", .{}) catch return -1;
    return 0;
}

export fn PutChar(u: u8) i64 {
    const stdout = std.io.getStdOut().writer();
    stdout.print("{c}", .{u}) catch return -1;
    return 0;
}
export fn PutNum(n: i64) i64 {
    const stdout = std.io.getStdOut().writer();
    stdout.print("{}", .{n}) catch return -1;
    return 0;
}

export fn PutNumHex(n: i64) i64 {
    const stdout = std.io.getStdOut().writer();
    stdout.print("0x{X}", .{n}) catch return -1;
    return 0;
}

export fn PutNumBin(n: i64) i64 {
    const stdout = std.io.getStdOut().writer();
    stdout.print("{b}", .{n}) catch return -1;
    return 0;
}

export fn InputLine() [*]i64 {
    var general_purpose_allocator = std.heap.GeneralPurposeAllocator(.{}){};
    const gpa = &general_purpose_allocator.allocator;
    const stdin = std.io.getStdIn().reader();
    const output = stdin.readUntilDelimiterAlloc(gpa, '\n', 10000) catch unreachable;
    defer gpa.free(output);
    var mem = gpa.alloc(i64, output.len + 2) catch unreachable;
    {
        var index: usize = 0;
        while (index < output.len) : (index += 1) {
            mem[index + 2] = @intCast(i64, output[index]);
        }
        mem[0] = @bitCast(i64, @ptrToInt(&mem));
        mem[1] = @bitCast(i64, output.len);
    }
    return mem.ptr;
}

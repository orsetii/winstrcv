# winstrcv - A program to convert windbg structs into Rust struct source code.


Currently all pointers are implemented as `mut* std::os::raw::c_void`, and results can be casted around (unsafely) to maintain interporability.

There is also no way of using bitfields in Rust, and makes using structs that have bitfields, impossible, there are some implementations but none suitable for static struct types, as are being extracted here. Hence, you will see bitfields implemented as the static byte(s) that hold them, with appended comments explaining their position and size, for each bitfield.
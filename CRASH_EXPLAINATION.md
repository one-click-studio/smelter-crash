# Crash explanation

## Context

Smelter can randomly crash when CEF / Chromium is enabled and it uses around 2 GB of RAM:
```rs
Thread 43 "MemoryInfra" received signal SIGILL, Illegal instruction.
[Switching to Thread 0x7fff86000680 (LWP 2621)]
0x00007fffed050151 in OnMemoryDump () at ../../base/trace_event/malloc_dump_provider.cc:450
warning: 450    ../../base/trace_event/malloc_dump_provider.cc: No such file or directory
```

From our experience this usually occurs after 10 to 60 minutes.

## Explanation

The crash occurs in CEF, in the `ReportAppleAllocStats` function from `malloc_dump_provider.cc` (see [source code](https://chromium.googlesource.com/chromium/src.git/+/refs/tags/132.0.6834.83/base/trace_event/malloc_dump_provider.cc#452)).

This is part of a thread responsible for collecting memory usage statistics. On Linux, it relies on GLIBC's `mallinfo` or `mallinfo2`. Whether one or the other is used is determined during the build:
```c++
#if defined(__GLIBC__) && defined(__GLIBC_PREREQ)
#if __GLIBC_PREREQ(2, 33)
#define MALLINFO2_FOUND_IN_LIBC
  struct mallinfo2 info = mallinfo2();
#endif
#endif  // defined(__GLIBC__) && defined(__GLIBC_PREREQ)
#if !defined(MALLINFO2_FOUND_IN_LIBC)
  struct mallinfo info = mallinfo();
#endif
```

Smelter uses a pre-built version of CEF from Spotify that was built against GLIBC 2.2.5 and uses mallinfo().

The issue is, this is a deprecated function that returns the memory usage details as i32. These values are then used on lines 200 and 201 of `malloc_dump_provider.cc`:
```c++
  *total_virtual_size += checked_cast<size_t>(info.arena + info.hblkhd);
  size_t total_allocated_size = checked_cast<size_t>(info.uordblks);
```

And `checked_cast` will crash if the values passed are < 0.

## Consequence

This means depending on the memory usage of the application, it will crash if:
- **arena + hblkhd** > i32::MAX
- **uordblks** > i32::MAX

This is tricky as **arena** or **hblkhd** can overflow themselves, but it's fine as long as their sum is >= 0. And most memory stats collection is already disabled from CEF in Smelter. But it can still occasionally run.


## Proposed patch

We would use a CEF build made against more recent libraries.
But as a simpler solution, we decided to build a custom .so library that overrides mallinfo by being loaded first with LD_PRELOAD. This version ensures the values returned will not overflow when used by CEF.

See [mallinfo-override](./mallinfo-override).
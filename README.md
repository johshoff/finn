Finn
====

_Finn_ is a reimplementation of the _find_ command found on various \*nix operating systems.
It's uses multiple threads to parallelize the workload as well as dispatch multiple
IO requests, which the OS might take advantage of.

Compiling
---------

Compile like

    cargo build --release

And run the command like

    target/release/finn main.rs

TODO
----

- Better error handling. Most importantly, it shouldn't panic in worker threads.
- Better handling of early return from threads. The two `tx` clones aren't very nice.
- Documentation
- Testing
- Support more search patterns (time, size, ...)

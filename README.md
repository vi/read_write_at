# read_write_at

Abstraction of a file- or block derive-like object, data from/to which can be read/written at offsets.

There are alreay some analogues of those traits, including in libstd.
But they are either platform-specific or tied to implementation of some algorithm.

This crate focuses on the abstraction itself, providing mostly wrappers and helper functions.

Traits are given in two varieties: with mutable `&mut self` and immutable `&self` methods.

libstd's platform-specific FileExt traits are forwarded for std::fs::File.

There is a generic wrapper for using `Read+Seek` or `Read+Write+Seek` objects

Immutable version of traits are implemented for `RefCell`s or `Mutex`s over mutable versions.
You may need to use `DerefWrapper` it you use trait ojects although.

TODO:

* `parking_lot` integration?
* vectored IO
* async?
* reading to uninitialized buffers?
* `bytes` crate intergration?

License: MIT/Apache-2.0

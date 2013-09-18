=======
bitfifo
=======

A fifo data structure, written in [rust](http://rust-lang.org), for
efficiently pushing/popping different arbitrary sizes of bits.

Disclaimers
===========

I'm new to rust.  If you have suggestions or patches, please file tickets!

Installation
============

    ```
    $ git clone 'https://github.com/nejucomo/rust-bitfifo'
    $ cd rust-bitfifo
    $ rust test
    $ rust build
    ```

Goals
=====

* To learn rust:

  + Get feedback on the style and conventions, learn the standard style.
  + Get feedback on correctness, performance, etc...
  + Make a pure rust library useful for others.
  + Figure out if std or extra already has sufficient features to obviate
    this library.  (I don't think bitvectors work well for fifo behavior.)
  + Did I mention rust is awesome?  It's quickly becoming my favorite
    systems language, and maybe favorite language across the board.

* Efficiency:

  + Storage:

    * `O(B)` storage space, where `B` is number of bits stored.
    * Smallish storage constant overhead (1 bit per bit stored, plus data structure overhead).

  + Time:

    * `O(1)` push, except for a possible ringbuffer allocation, for fixed size int types.
    * `O(1)` pop, for fixed size int types.
    * `O(log(I))` push, except for possible ringbuffer allocation, for bigints.
    * `O(log(I))` pop, except for possible ringbuffer allocation, for bigints.

* Engineering:

  + Nice API design.
  + Good unittest coverage.
  + Profiling, someday.

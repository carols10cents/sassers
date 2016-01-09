# Sassers

A Sass compiler written natively in Rust.

[![Build Status](https://travis-ci.org/carols10cents/sassers.svg?branch=master)](https://travis-ci.org/carols10cents/sassers)

**Incomplete!!!!** To see current progress:

* Clone this project and `cargo build`
* Clone [sass-spec](https://github.com/sass/sass-spec/) and get ruby and all that good stuff
* In sass-spec, run `./sass-spec.rb -c '/path/to/your/sassers/executable'

Last run I did was 335 passing and 4164 failing out of 4499 tests. So yeah, REALLY NOT DONE YET.

Progress bar: [=-------------------]

Sassers follows [Sentimental Versioning](http://sentimentalversioning.org/).

## License

MIT. See LICENSE.

## Optimizations

* Pass `T: Write` down into methods instead of returning String all the time to avoid allocating so many times

## TODO

* Compare speed/memory usage to libsass
* Profile if it's significantly worse than libsass and fix
* Abstract variable/parameter HashMaps into a `context` or `binding` object with nice insertion and accessing methods

## Useful debugging incantation

To get debugging statements and run a particular test:

```
$ RUST_LOG=sassers=debug cargo test -- --nocapture evaluator::tests::it_subtitutes_variable_values
```

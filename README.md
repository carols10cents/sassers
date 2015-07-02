# Sassers

A Sass compiler written natively in Rust.

**Incomplete!!!!** To see current progress:

* Clone this project and `cargo build`
* Clone [sass-spec](https://github.com/sass/sass-spec/) and get ruby and all that good stuff
* In sass-spec, run `./sass-spec.rb -c '/path/to/your/sassers/executable'

Last run I did was 299 passing and 4200 failing out of 4499 tests. So yeah, REALLY NOT DONE YET.

Progress bar: [=-------------------]

Sassers follows [Sentimental Versioning](http://sentimentalversioning.org/).

## License

MIT. See LICENSE.

## Optimizations

* Pass `T: Write` down into methods instead of returning String all the time to avoid allocating so many times

## TODO

* Travis running sass-spec
* Compare speed/memory usage to libsass
* Profile if it's significantly worse than libsass and fix
* Shell script for running sets of sass-specs

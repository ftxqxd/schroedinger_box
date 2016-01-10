`SchroedingerBox`
=================

A box that contains many values, but collapses into one when opened (read from)
for the first time.

Example
-------

```rust
let cat_is_alive = SchroedingerBox::new(vec![true, false]);
// Here the cat is both dead and alive, but when we observe it...
let state = *cat_is_alive;
// ...it collapses into one of the possible states with equal probability.
assert_eq!(state, *cat_is_alive);
```
## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

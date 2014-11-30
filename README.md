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
// ...it collapses into one state.
assert_eq!(state, *cat_is_alive);
```

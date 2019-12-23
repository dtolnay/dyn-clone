Clone trait that is object-safe
===============================

[![Build Status](https://api.travis-ci.org/dtolnay/objekt.svg?branch=master)](https://travis-ci.org/dtolnay/objekt)
[![Latest Version](https://img.shields.io/crates/v/objekt.svg)](https://crates.io/crates/objekt)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/objekt/0.1/objekt/)

This crate provides a `DynClone` trait that can be used in trait objects, and a
`clone_box` function that can clone any sized or dynamically sized
implementation of `DynClone`. Types that implement the standard library's
[`std::clone::Clone`] trait are automatically usable by a `DynClone` trait
object.

[`std::clone::Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html

The signature of `clone_box` is:

```rust
fn clone_box<T>(t: &T) -> Box<T>
where
    T: ?Sized + DynClone
```

## Example

```rust
use objekt::DynClone;

trait MyTrait: DynClone {
    fn recite(&self);
}

impl MyTrait for String {
    fn recite(&self) {
        println!("{} ♫", self);
    }
}

fn main() {
    let line = "The slithy structs did gyre and gimble the namespace";

    // Build a trait object holding a String.
    // This requires String to implement MyTrait and std::clone::Clone.
    let x: Box<dyn MyTrait> = Box::new(String::from(line));

    x.recite();

    // The type of x2 is a Box<dyn MyTrait> cloned from x.
    let x2 = objekt::clone_box(&*x);

    x2.recite();
}
```

This crate includes a macro for generating the implementation `impl
std::clone::Clone for Box<dyn MyTrait>` in terms of `objekt::clone_box`:

```rust
// As before.
trait MyTrait: DynClone {
    /* ... */
}

objekt::clone_trait_object!(MyTrait);

// Now data structures containing Box<dyn MyTrait> can derive Clone:
#[derive(Clone)]
struct Container {
    trait_object: Box<dyn MyTrait>,
}
```

Check out the [objekt-clonable] crate which provides the same Clone impl for
`Box<dyn MyTrait>` in a more concise attribute form.

[objekt-clonable]: https://github.com/kardeiz/objekt-clonable

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

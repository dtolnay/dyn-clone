//! [![github]](https://github.com/dtolnay/dyn-clone)&ensp;[![crates-io]](https://crates.io/crates/dyn-clone)&ensp;[![docs-rs]](https://docs.rs/dyn-clone)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! This crate provides a [`DynClone`] trait that can be used in trait objects,
//! and a [`clone_box`] function that can clone any sized or dynamically sized
//! implementation of `DynClone`. Types that implement the standard library's
//! [`std::clone::Clone`] trait are automatically usable by a `DynClone` trait
//! object.
//!
//! [`DynClone`]: trait.DynClone.html
//! [`clone_box`]: fn.clone_box.html
//! [`std::clone::Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
//!
//! # Example
//!
//! ```
//! use dyn_clone::DynClone;
//!
//! trait MyTrait: DynClone {
//!     fn recite(&self);
//! }
//!
//! impl MyTrait for String {
//!     fn recite(&self) {
//!         println!("{} ♫", self);
//!     }
//! }
//!
//! fn main() {
//!     let line = "The slithy structs did gyre and gimble the namespace";
//!
//!     // Build a trait object holding a String.
//!     // This requires String to implement MyTrait and std::clone::Clone.
//!     let x: Box<dyn MyTrait> = Box::new(String::from(line));
//!
//!     x.recite();
//!
//!     // The type of x2 is a Box<dyn MyTrait> cloned from x.
//!     let x2 = dyn_clone::clone_box(&*x);
//!
//!     x2.recite();
//! }
//! ```
//!
//! This crate includes a macro for concisely implementing `impl
//! std::clone::Clone for Box<dyn MyTrait>` in terms of `dyn_clone::clone_box`.
//!
//! ```
//! # use dyn_clone::DynClone;
//! #
//! // As before.
//! trait MyTrait: DynClone {
//!     /* ... */
//! }
//!
//! dyn_clone::clone_trait_object!(MyTrait);
//!
//! // Now data structures containing Box<dyn MyTrait> can derive Clone:
//! #[derive(Clone)]
//! struct Container {
//!     trait_object: Box<dyn MyTrait>,
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/dyn_clone/1.0.6")]
#![no_std]
#![allow(clippy::missing_panics_doc)]

extern crate alloc;

use crate::sealed::{Private, Sealed};

#[macro_use]
mod macros;

#[doc(hidden)]
pub mod private {
    pub use alloc::boxed::Box;
    pub use core::clone::Clone;
    pub use core::marker::{Send, Sync};
}

mod sealed {
    pub trait Sealed {}
    impl<T: Clone> Sealed for T {}
    pub struct Private;
}

/// This trait is implemented by any type that implements [`std::clone::Clone`].
///
/// [`std::clone::Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
pub trait DynClone: Sealed {
    // Not public API
    #[doc(hidden)]
    fn __clone_box(&self, _: Private) -> *mut ();
}

use alloc::boxed::Box;

pub fn clone<T>(t: &T) -> T
where
    T: DynClone,
{
    unsafe { *Box::from_raw(<T as DynClone>::__clone_box(t, Private) as *mut T) }
}

pub fn clone_box<T>(t: &T) -> Box<T>
where
    T: ?Sized + DynClone,
{
    let mut fat_ptr = t as *const T;
    unsafe {
        let data_ptr = &mut fat_ptr as *mut *const T as *mut *mut ();
        assert_eq!(*data_ptr as *const (), t as *const T as *const ());
        *data_ptr = <T as DynClone>::__clone_box(t, Private);
    }
    unsafe { Box::from_raw(fat_ptr as *mut T) }
}

impl<T> DynClone for T
where
    T: Clone,
{
    fn __clone_box(&self, _: Private) -> *mut () {
        Box::into_raw(Box::new(self.clone())) as *mut ()
    }
}

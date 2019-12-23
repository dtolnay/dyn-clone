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

#![doc(html_root_url = "https://docs.rs/dyn_clone/0.1.2")]

#[macro_use]
mod macros;

#[doc(hidden)]
pub use std as private;

/// This trait is implemented by any type that implements [`std::clone::Clone`].
///
/// [`std::clone::Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
pub trait DynClone {
    // Not public API
    #[doc(hidden)]
    unsafe fn clone_box(&self) -> *mut ();
}

pub fn clone<T>(t: &T) -> T
where
    T: DynClone,
{
    unsafe {
        *Box::from_raw(<T as DynClone>::clone_box(t) as *mut T)
    }
}

pub fn clone_box<T>(t: &T) -> Box<T>
where
    T: ?Sized + DynClone,
{
    let mut fat_ptr = t as *const T;
    unsafe {
        let data_ptr = &mut fat_ptr as *mut *const T as *mut *mut ();
        assert_eq!(*data_ptr as *const (), t as *const T as *const ());
        *data_ptr = <T as DynClone>::clone_box(t);
    }
    unsafe {
        Box::from_raw(fat_ptr as *mut T)
    }
}

impl<T> DynClone for T
where
    T: std::clone::Clone,
{
    unsafe fn clone_box(&self) -> *mut () {
        Box::into_raw(Box::new(self.clone())) as *mut ()
    }
}

#[cfg(test)]
mod tests {
    use super::DynClone;
    use std::fmt::{self, Display};
    use std::sync::{Arc, Mutex};

    struct Log {
        id: u64,
        events: Arc<Mutex<Vec<String>>>,
    }

    impl Clone for Log {
        fn clone(&self) -> Self {
            Log {
                id: self.id + 1,
                events: self.events.clone(),
            }
        }
    }

    impl Display for Log {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "id={}", self.id)
        }
    }

    impl Drop for Log {
        fn drop(&mut self) {
            self.events.lock().unwrap().push(format!("dropping {}", self))
        }
    }

    #[test]
    fn clone_sized() {
        let arc = Arc::new(0);
        assert_eq!(Arc::strong_count(&arc), 1);

        let c = crate::clone(&arc);
        assert_eq!(Arc::strong_count(&arc), 2);
        drop(c);
        assert_eq!(Arc::strong_count(&arc), 1);
    }

    #[test]
    fn clone_trait_object() {
        trait MyTrait: Display + Sync + DynClone {}

        impl MyTrait for Log {}

        let events = Arc::new(Mutex::new(Vec::new()));
        let mut expected = Vec::new();
        {
            let b11: Box<dyn MyTrait> = Box::new(Log {
                id: 11,
                events: events.clone(),
            });
            let b12 = crate::clone_box(&*b11);
            assert_eq!(b11.to_string(), "id=11");
            assert_eq!(b12.to_string(), "id=12");
            expected.push("dropping id=12".to_owned());
            expected.push("dropping id=11".to_owned());
        }
        assert_eq!(*events.lock().unwrap(), expected);
    }
}

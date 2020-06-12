/// Implement the standard library `Clone` for a trait object that has
/// `DynClone` as a supertrait.
///
/// ```
/// use dyn_clone::DynClone;
///
/// trait MyTrait: DynClone {
///     /* ... */
/// }
///
/// dyn_clone::clone_trait_object!(MyTrait);
///
/// // Now data structures containing Box<dyn MyTrait> can derive Clone.
/// #[derive(Clone)]
/// struct Container {
///     trait_object: Box<dyn MyTrait>,
/// }
/// ```
///
/// The macro supports traits that have type parameters and/or `where` clauses.
///
/// ```
/// use dyn_clone::DynClone;
/// use std::io::Read;
///
/// trait Difficult<R>: DynClone where R: Read {
///     /* ... */
/// }
///
/// dyn_clone::clone_trait_object!(<R> Difficult<R> where R: Read);
/// ```
#[macro_export]
macro_rules! clone_trait_object {
    ($($path:tt)+) => {
        $crate::__internal_clone_trait_object!(begin $($path)+);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_clone_trait_object {
    // Invocation started with `<`, parse generics.
    (begin < $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(generics () () $($rest)*);
    };

    // Invocation did not start with `<`.
    (begin $first:tt $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(path () ($first) $($rest)*);
    };

    // End of generics.
    (generics ($($generics:tt)*) () > $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(path ($($generics)*) () $($rest)*);
    };

    // Generics open bracket.
    (generics ($($generics:tt)*) ($($brackets:tt)*) < $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(generics ($($generics)* <) ($($brackets)* <) $($rest)*);
    };

    // Generics close bracket.
    (generics ($($generics:tt)*) (< $($brackets:tt)*) > $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(generics ($($generics)* >) ($($brackets)*) $($rest)*);
    };

    // Token inside of generics.
    (generics ($($generics:tt)*) ($($brackets:tt)*) $first:tt $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(generics ($($generics)* $first) ($($brackets)*) $($rest)*);
    };

    // End with `where` clause.
    (path ($($generics:tt)*) ($($path:tt)*) where $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(impl ($($generics)*) ($($path)*) ($($rest)*));
    };

    // End without `where` clause.
    (path ($($generics:tt)*) ($($path:tt)*)) => {
        $crate::__internal_clone_trait_object!(impl ($($generics)*) ($($path)*) ());
    };

    // Token inside of path.
    (path ($($generics:tt)*) ($($path:tt)*) $first:tt $($rest:tt)*) => {
        $crate::__internal_clone_trait_object!(path ($($generics)*) ($($path)* $first) $($rest)*);
    };

    // The impl.
    (impl ($($generics:tt)*) ($($path:tt)*) ($($bound:tt)*)) => {
        impl<'clone, $($generics)*> $crate::private_core::clone::Clone for $crate::private_alloc::boxed::Box<dyn $($path)* + 'clone> where $($bound)* {
            fn clone(&self) -> Self {
                $crate::clone_box(&**self)
            }
        }
    };
}

// TEST ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::DynClone;
    use alloc::boxed::Box;

    fn assert_clone<T: Clone>() {}

    #[test]
    fn test_plain() {
        trait Trait: DynClone {}

        clone_trait_object!(Trait);

        assert_clone::<Box<dyn Trait>>();
    }

    #[test]
    fn test_type_parameter() {
        trait Trait<T>: DynClone {}

        clone_trait_object!(<T> Trait<T>);

        assert_clone::<Box<dyn Trait<u32>>>();
    }

    #[test]
    fn test_generic_bound() {
        trait Trait<T: PartialEq<T>, U>: DynClone {}

        clone_trait_object!(<T: PartialEq<T>, U> Trait<T, U>);

        assert_clone::<Box<dyn Trait<u32, ()>>>();
    }

    #[test]
    fn test_where_clause() {
        trait Trait<T>: DynClone where T: Clone {}

        clone_trait_object!(<T> Trait<T> where T: Clone);

        assert_clone::<Box<dyn Trait<u32>>>();
    }
}

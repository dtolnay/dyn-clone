/// Implement the standard library `Clone` for a trait object that has
/// `objekt::Clone` as a supertrait.
///
/// ```edition2018
/// trait MyTrait: objekt::Clone {
///     /* ... */
/// }
///
/// objekt::clone_trait_object!(MyTrait);
///
/// // Now data structures containing Box<MyTrait> can derive Clone.
/// #[derive(Clone)]
/// struct Container {
///     trait_object: Box<MyTrait>,
/// }
/// ```
///
/// The macro supports traits that have type parameters and/or `where` clauses.
///
/// ```edition2018
/// use std::io::Read;
///
/// trait Difficult<R>: objekt::Clone where R: Read {
///     /* ... */
/// }
///
/// objekt::clone_trait_object!(<R> Difficult<R> where R: Read);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! clone_trait_object {
    ($($path:tt)+) => {
        __internal_clone_trait_object!(begin $($path)+);
    };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! __internal_clone_trait_object {
    // Invocation started with `<`, parse generics.
    (begin < $($rest:tt)*) => {
        __internal_clone_trait_object!(generics () () $($rest)*);
    };

    // Invocation did not start with `<`.
    (begin $first:tt $($rest:tt)*) => {
        __internal_clone_trait_object!(path () ($first) $($rest)*);
    };

    // End of generics.
    (generics ($($generics:tt)*) () > $($rest:tt)*) => {
        __internal_clone_trait_object!(path ($($generics)*) () $($rest)*);
    };

    // Generics open bracket.
    (generics ($($generics:tt)*) ($($brackets:tt)*) < $($rest:tt)*) => {
        __internal_clone_trait_object!(generics ($($generics)* <) ($($brackets)* <) $($rest)*);
    };

    // Generics close bracket.
    (generics ($($generics:tt)*) (< $($brackets:tt)*) > $($rest:tt)*) => {
        __internal_clone_trait_object!(generics ($($generics)* >) ($($brackets)*) $($rest)*);
    };

    // Token inside of generics.
    (generics ($($generics:tt)*) ($($brackets:tt)*) $first:tt $($rest:tt)*) => {
        __internal_clone_trait_object!(generics ($($generics)* $first) ($($brackets)*) $($rest)*);
    };

    // End with `where` clause.
    (path ($($generics:tt)*) ($($path:tt)*) where $($rest:tt)*) => {
        __internal_clone_trait_object!(impl ($($generics)*) ($($path)*) ($($rest)*));
    };

    // End without `where` clause.
    (path ($($generics:tt)*) ($($path:tt)*)) => {
        __internal_clone_trait_object!(impl ($($generics)*) ($($path)*) ());
    };

    // Token inside of path.
    (path ($($generics:tt)*) ($($path:tt)*) $first:tt $($rest:tt)*) => {
        __internal_clone_trait_object!(path ($($generics)*) ($($path)* $first) $($rest)*);
    };

    // The impl.
    (impl ($($generics:tt)*) ($($path:tt)*) ($($bound:tt)*)) => {
        impl<'clone, $($generics)*> $crate::private::clone::Clone for $crate::private::boxed::Box<$($path)* + 'clone> where $($bound)* {
            fn clone(&self) -> Self {
                $crate::clone_box(&**self)
            }
        }
    };
}

// TEST ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    fn assert_clone<T: Clone>() {}

    #[test]
    fn test_plain() {
        trait Trait: ::Clone {}

        clone_trait_object!(Trait);

        assert_clone::<Box<Trait>>();
    }

    #[test]
    fn test_type_parameter() {
        trait Trait<T>: ::Clone {}

        clone_trait_object!(<T> Trait<T>);

        assert_clone::<Box<Trait<u32>>>();
    }

    #[test]
    fn test_generic_bound() {
        trait Trait<T: PartialEq<T>, U>: ::Clone {}

        clone_trait_object!(<T: PartialEq<T>, U> Trait<T, U>);

        assert_clone::<Box<Trait<u32, ()>>>();
    }

    #[test]
    fn test_where_clause() {
        trait Trait<T>: ::Clone where T: Clone {}

        clone_trait_object!(<T> Trait<T> where T: Clone);

        assert_clone::<Box<Trait<u32>>>();
    }
}

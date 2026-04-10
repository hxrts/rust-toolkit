//! Generic support traits for effect vocabularies and concrete handlers.
//!
//! This crate contains the portable backing traits used by the toolkit's
//! `#[effect_trait]` and `#[effect_handler]` proc macros.

#![forbid(unsafe_code)]

mod sealed {
    pub trait EffectSealed {}

    impl<T> EffectSealed for T where
        T: ?Sized + crate::__private::EffectDefinition + Send + Sync + 'static
    {
    }
}

/// Marker trait for abstract effect vocabularies.
///
/// A trait satisfies `Effect` when an `#[effect_trait]`-style proc macro emits
/// the hidden `EffectDefinition` witness for its trait object.
pub trait Effect: sealed::EffectSealed + Send + Sync + 'static {}

impl<T> Effect for T where
    T: ?Sized + crate::__private::EffectDefinition + Send + Sync + 'static
{
}

/// Marker trait for concrete handlers of one effect vocabulary.
pub trait EffectHandler<E>: Send + Sync + 'static
where
    E: ?Sized + Effect,
    Self: crate::__private::HandlerDefinition<E>,
{
}

impl<T, E> EffectHandler<E> for T
where
    T: ?Sized + Send + Sync + 'static,
    E: ?Sized + Effect,
    T: crate::__private::HandlerDefinition<E>,
{
}

/// Hidden support items used by the proc macros.
#[doc(hidden)]
pub mod __private {
    use core::marker::PhantomData;

    pub trait EffectDefinition {}

    pub trait HandlerDefinition<E: ?Sized> {}

    // fn() -> (*const T, *const E) makes both T and E invariant without
    // introducing the Send/Sync implications of raw pointers.
    pub struct HandlerToken<T: ?Sized, E: ?Sized>(
        pub PhantomData<fn() -> (*const T, *const E)>,
    );
}

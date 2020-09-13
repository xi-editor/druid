use crate::optics::{lens, prism, Lens};

use std::marker::PhantomData;

// TODO: rename that Prism to AffineTraversal, and bring it here?
// or leave it there as PartialPrism? (or both?)
// TODO: a trait re-export really shouldn't be happening..
// TODO: rename Traversal to AffineTraversal?
// since affine means 0-or-1 (same as Option<T>), and Traversal
// may be 0-or-1-or-many.
pub use crate::optics::Prism as Traversal;

pub use then_affine_traversal::ThenAffineTraversal;

mod then_affine_traversal {
    use super::{lens, prism};

    pub trait ThenAffineTraversal<
        Other,
        A: ?Sized,
        B: ?Sized,
        C: ?Sized,
        OriginPhantom,
        DestinationPhantom,
    >
    {
        type Target;
        fn then(self, other: Other) -> Self::Target;
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct LensUnit;

    #[derive(Clone, Debug, PartialEq)]
    pub struct PrismUnit;

    /// Compose a `Lens<A, B>` with a `Lens<B, C>` to produce a `Lens<A, C>`.
    ///
    /// ```
    /// # use druid::{*, optics::traversal::ThenAffineTraversal};
    /// struct Foo { x: (u32, bool) }
    /// let lens = lens!(Foo, x).then(lens!((u32, bool), 1));
    /// assert_eq!(lens.get(&Foo { x: (0, true) }), true);
    /// ```
    impl<L1, L2, A, B, C> ThenAffineTraversal<L2, A, B, C, LensUnit, LensUnit> for L1
    where
        A: ?Sized,
        B: ?Sized,
        C: ?Sized,
        L1: lens::Lens<A, B>, /*+ Sized*/
        L2: lens::Lens<B, C>,
        // C: Sized,
        // Self: Sized,
    {
        type Target = lens::Then<L1, L2, B>;
        fn then(self, lens: L2) -> Self::Target {
            lens::Then::new(self, lens)
        }
    }

    /// Compose a `Lens<A, B>` with a `Prism<B, C>` to produce a `Prism<A, C>`.
    impl<L1, P2, A, B, C> ThenAffineTraversal<P2, A, B, C, LensUnit, PrismUnit> for L1
    where
        A: ?Sized,
        B: ?Sized,
        C: ?Sized,
        L1: lens::Lens<A, B>, /* + Sized*/
        P2: prism::Prism<B, C>,
        // C: Sized,
        // Self: Sized,
    {
        type Target = super::ThenAfterLens<Self, P2, B>;
        fn then(self, prism: P2) -> Self::Target {
            super::ThenAfterLens::new(self, prism)
        }
    }

    /// Compose a `Prism<A, B>` with a `Lens<B, C>` to produce a `Prism<A, C>`.
    impl<P1, L2, A, B, C> ThenAffineTraversal<L2, A, B, C, PrismUnit, LensUnit> for P1
    where
        A: ?Sized,
        B: ?Sized,
        C: ?Sized,
        P1: prism::Prism<A, B>, /*+ Sized*/
        L2: lens::Lens<B, C>,
        // C: Sized,
        // Self: Sized,
    {
        type Target = super::ThenLens<P1, L2, B>;
        fn then(self, lens: L2) -> Self::Target {
            super::ThenLens::new(self, lens)
        }
    }

    /// Compose a `Prism<A, B>` with a `Prism<B, C>` to produce a `Prism<A, C>`.
    impl<P1, P2, A, B, C> ThenAffineTraversal<P2, A, B, C, PrismUnit, PrismUnit> for P1
    where
        A: ?Sized,
        B: ?Sized,
        C: ?Sized,
        P1: prism::Prism<A, B>, /*+ Sized*/
        P2: prism::Prism<B, C>,
        // C: Sized,
        // Self: Sized,
    {
        type Target = prism::Then<P1, P2, B>;
        fn then(self, prism: P2) -> Self::Target {
            prism::Then::new(self, prism)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LensWrap<L> {
    inner: L,
}

impl<L> LensWrap<L> {
    pub fn new<S: ?Sized, A: ?Sized>(inner: L) -> Self
    where
        L: Lens<S, A>,
    {
        Self { inner }
    }
}

impl<L, S, A> Traversal<S, A> for LensWrap<L>
where
    S: ?Sized,
    A: ?Sized,
    L: Lens<S, A>,
{
    fn with<V, F: FnOnce(&A) -> V>(&self, data: &S, f: F) -> Option<V> {
        Some(self.inner.with(data, f))
    }

    fn with_mut<V, F: FnOnce(&mut A) -> V>(&self, data: &mut S, f: F) -> Option<V> {
        Some(self.inner.with_mut(data, f))
    }
}

#[derive(Debug, Copy, PartialEq)]
pub struct ThenLens<P1, L2, B: ?Sized> {
    left: P1,
    right: L2,
    _marker: PhantomData<B>,
}

impl<P1, L2, B: ?Sized> ThenLens<P1, L2, B> {
    pub fn new<A: ?Sized, C: ?Sized>(left: P1, right: L2) -> Self
    where
        P1: Traversal<A, B>,
        L2: Lens<B, C>,
    {
        Self {
            left,
            right,
            _marker: PhantomData,
        }
    }
}

impl<P1, L2, A, B, C> Traversal<A, C> for ThenLens<P1, L2, B>
where
    A: ?Sized,
    B: ?Sized,
    C: ?Sized,
    P1: Traversal<A, B>,
    L2: Lens<B, C>,
{
    fn with<V, F: FnOnce(&C) -> V>(&self, data: &A, f: F) -> Option<V> {
        self.left.with(data, |b| self.right.with(b, f))
    }

    fn with_mut<V, F: FnOnce(&mut C) -> V>(&self, data: &mut A, f: F) -> Option<V> {
        self.left.with_mut(data, |b| self.right.with_mut(b, f))
    }
}

impl<P1: Clone, L2: Clone, B> Clone for ThenLens<P1, L2, B> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            _marker: PhantomData,
        }
    }
}

// TODO: decide if this should exist..
impl<P1, L2, A, B, C> prism::Replace<A, C> for ThenLens<P1, L2, B>
where
    A: ?Sized + Default,
    B: ?Sized + Default,
    C: Sized + Clone,
    P1: prism::Prism<A, B> + prism::Replace<A, B>,
    L2: Lens<B, C>,
{
    fn replace<'a>(&self, base: &'a mut A, v: C) -> &'a mut A
    where
        A: Sized,
    {
        self.left.replace(base, {
            // build B -> C from scratch
            let mut new_b = B::default();
            let () = self.right.with_mut(&mut new_b, |c| *c = v);

            new_b
        })
    }
}

#[derive(Debug, Copy, PartialEq)]
pub struct AndLens<P1, L2, B: ?Sized> {
    left: P1,
    right: L2,
    _marker: PhantomData<B>,
}

impl<P1, L2, B: ?Sized> AndLens<P1, L2, B> {
    pub fn new<A: ?Sized, C: ?Sized>(left: P1, right: L2) -> Self
    where
        P1: Traversal<A, B>,
        L2: Lens<A, C>,
    {
        Self {
            left,
            right,
            _marker: PhantomData,
        }
    }
}

impl<P1, L2, A, B, C> Traversal<A, C> for AndLens<P1, L2, B>
where
    A: ?Sized,
    B: ?Sized,
    C: ?Sized,
    P1: Traversal<A, B>,
    L2: Lens<A, C>,
{
    fn with<V, F: FnOnce(&C) -> V>(&self, data: &A, f: F) -> Option<V> {
        self.left
            .with(data, |_b| ())
            .and(Some(self.right.with(data, f)))
    }

    fn with_mut<V, F: FnOnce(&mut C) -> V>(&self, data: &mut A, f: F) -> Option<V> {
        self.left
            .with_mut(data, |_b| ())
            .and(Some(self.right.with_mut(data, f)))
    }
}

impl<P1: Clone, L2: Clone, B> Clone for AndLens<P1, L2, B> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Copy, PartialEq)]
pub struct ThenAfterLens<L1, P2, B: ?Sized> {
    left: L1,
    right: P2,
    _marker: PhantomData<B>,
}

impl<L1, P2, B: ?Sized> ThenAfterLens<L1, P2, B> {
    pub fn new<A: ?Sized, C: ?Sized>(left: L1, right: P2) -> Self
    where
        L1: Lens<A, B>,
        P2: Traversal<B, C>,
    {
        Self {
            left,
            right,
            _marker: PhantomData,
        }
    }
}

impl<L1, P2, A, B, C> Traversal<A, C> for ThenAfterLens<L1, P2, B>
where
    A: ?Sized,
    B: ?Sized,
    C: ?Sized,
    L1: Lens<A, B>,
    P2: Traversal<B, C>,
{
    fn with<V, F: FnOnce(&C) -> V>(&self, data: &A, f: F) -> Option<V> {
        self.left.with(data, |b| self.right.with(b, f))
    }

    fn with_mut<V, F: FnOnce(&mut C) -> V>(&self, data: &mut A, f: F) -> Option<V> {
        self.left.with_mut(data, |b| self.right.with_mut(b, f))
    }
}

impl<L1: Clone, P2: Clone, B> Clone for ThenAfterLens<L1, P2, B> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            _marker: PhantomData,
        }
    }
}

// TODO: decide if this should exist..
impl<L1, P2, A, B, C> prism::Replace<A, C> for ThenAfterLens<L1, P2, B>
where
    A: ?Sized + Default,
    B: ?Sized + Default,
    C: Sized + Clone,
    L1: Lens<A, B>,
    P2: Traversal<B, C> + prism::Replace<B, C>,
{
    /// Given the matching path of `A` -> `B` -> `C`,
    /// it is guaranteed that `B` will end up matching
    /// to `C`.
    ///
    /// It only forwards the replacement into `B` -> `C`.
    fn replace<'a>(&self, data: &'a mut A, v: C) -> &'a mut A {
        #[allow(clippy::unused_unit)]
        let () = self.left.with_mut(
            data,
            // A -> B was already set
            // only replaces B -> C
            // (as A -> B is already set)
            |b| {
                self.right.replace(b, v);
                ()
            },
        );
        data
    }
}

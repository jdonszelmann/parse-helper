use core::mem;
use std::{
    marker::PhantomData,
    ops::{Index, Range},
};

use crate::{Byte, ParseHelper};

mod private {
    use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
    use std::ops::Index;

    use super::Mark;
    use crate::Byte;

    pub trait SliceRange<'a, B, T: ?Sized> {
        type RangeTy;

        fn slice(&self, inp: &'a T) -> &'a <T as Index<Self::RangeTy>>::Output
        where
            T: Index<Self::RangeTy>;
    }

    // NOTE: the items for which this trait is implemented are carefully
    // chosen to avoid allowing for slicing in the middle of unicode codepoints.

    impl<'a, B, T: ?Sized> SliceRange<'a, B, T> for Range<Mark<B>> {
        type RangeTy = Range<usize>;

        fn slice(&self, inp: &'a T) -> &'a <T as Index<Self::RangeTy>>::Output
        where
            T: Index<Self::RangeTy>,
        {
            &inp[self.start.byte_position..self.end.byte_position]
        }
    }
    impl<'a, B, T: ?Sized> SliceRange<'a, B, T> for RangeFrom<Mark<B>> {
        type RangeTy = RangeFrom<usize>;
        fn slice(&self, inp: &'a T) -> &'a <T as Index<Self::RangeTy>>::Output
        where
            T: Index<Self::RangeTy>,
        {
            &inp[self.start.byte_position..]
        }
    }
    impl<'a, B, T: ?Sized> SliceRange<'a, B, T> for RangeFull {
        type RangeTy = RangeFull;
        fn slice(&self, inp: &'a T) -> &'a <T as Index<Self::RangeTy>>::Output
        where
            T: Index<Self::RangeTy>,
        {
            &inp[..]
        }
    }
    impl<'a, B, T: ?Sized> SliceRange<'a, B, T> for RangeTo<Mark<B>> {
        type RangeTy = RangeTo<usize>;
        fn slice(&self, inp: &'a T) -> &'a <T as Index<Self::RangeTy>>::Output
        where
            T: Index<Self::RangeTy>,
        {
            &inp[..self.end.byte_position]
        }
    }

    impl<'a, T: ?Sized> SliceRange<'a, Byte, T> for RangeToInclusive<Mark<Byte>> {
        type RangeTy = RangeToInclusive<usize>;
        fn slice(&self, inp: &'a T) -> &'a <T as Index<Self::RangeTy>>::Output
        where
            T: Index<Self::RangeTy>,
        {
            &inp[..=self.end.byte_position]
        }
    }
    impl<'a, T: ?Sized> SliceRange<'a, Byte, T> for RangeInclusive<Mark<Byte>> {
        type RangeTy = RangeInclusive<usize>;

        fn slice(&self, inp: &'a T) -> &'a <T as Index<Self::RangeTy>>::Output
        where
            T: Index<Self::RangeTy>,
        {
            &inp[self.start().byte_position..=self.end().byte_position]
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
/// Marks a position in an input stream.
pub struct Mark<B> {
    byte_position: usize,
    boundary: PhantomData<B>,
}

impl<B> Mark<B> {
    /// get the position in the input of this mark.
    pub fn byte_position(&self) -> usize {
        self.byte_position
    }
}

impl<'a, T: ?Sized, B> ParseHelper<'a, T, B> {
    /// Same as clone, but this can help show intent (together with
    /// [`restore_backup`](Self::restore_backup))
    pub fn create_backup(&self) -> Self {
        self.clone()
    }

    /// simply overwrites self. However, can be nice to show intent.
    pub fn restore_backup(&mut self, other: Self) {
        *self = other;
    }

    /// Creates a mark at the current position of the parse helper.
    ///
    /// Used in combination with [`slice`](Self::slice)
    pub fn mark(&self) -> Mark<B> {
        Mark {
            byte_position: self.byte_position,
            boundary: PhantomData,
        }
    }

    /// Slices a the input of a parse helper between two marks.
    ///
    /// ```
    /// use parse_helper::ParseHelper;
    ///
    /// let mut ph = ParseHelper::new_char_oriented("ab cd");
    /// let start = ph.mark();
    ///
    /// ph.accept_until_whitespace();
    /// ph.accept_one_or_more_whitespace();
    /// ph.accept_char('c');
    ///
    /// let end = ph.mark();
    ///
    /// assert_eq!(ph.slice(start..end), "ab c")
    /// ```
    ///
    /// Note that you can slice a [`Char`](crate::Char) oriented parse helper only using exclusive ranges
    /// (to not split utf8 codepoints accidentally), but [`Byte`] orieinted parse helpers can
    /// be slices using these ranges. This property is encoded in a sealed trait called `SliceRange`.
    pub fn slice<R>(&self, range: R) -> &'a <T as Index<R::RangeTy>>::Output
    where
        R: private::SliceRange<'a, B, T>,
        T: Index<R::RangeTy>,
    {
        // do not remove bounds checks for this (like make it unsafe), nothing guarantees
        // that the marks given are in-bounds. We only know that they're either char or byte
        // oriented correctly.
        range.slice(self.input)
    }

    /// Takes a closure as a parameter. Anything accepted within the closure is accepted as normal,
    /// but at the end you get back a subslice of the input of everything that was accepted in the
    /// closure.
    ///
    /// ```
    /// use parse_helper::ParseHelper;
    ///
    /// let mut ph = ParseHelper::new_char_oriented("ab cd");
    ///
    /// let (_, all_accepted) = ph.slice_accepted(|ph| {
    ///     ph.accept_until_whitespace();
    ///     ph.accept_one_or_more_whitespace();
    ///     ph.accept_char('c');
    /// });
    ///
    /// assert_eq!(all_accepted, "ab c");
    /// ```
    pub fn slice_accepted<P>(
        &mut self,
        closure: impl FnOnce(&mut Self) -> P,
    ) -> (P, &'a <T as Index<Range<usize>>>::Output)
    where
        T: Index<Range<usize>>,
    {
        let start = self.mark();
        let res = closure(self);
        let end = self.mark();

        (res, self.slice(start..end))
    }

    /// Most accept functions return an `Option`. This is like [`slice_accepted`]
    /// but the closure is supposed to return `Option<()>`. If it's `Some`, you get back the entire
    /// accepted range, if it's `None`, you get back `None`. This may sound like a very specific
    /// usecase, but it comes up a lot.
    ///
    /// This is also safer than [`slice_accepted`], because if None is returned, the parser resets
    /// to where it was before accepting anything
    ///
    /// [`slice_accepted`]: ParseHelper::slice_accepted
    ///
    /// ```
    /// use parse_helper::ParseHelper;
    ///
    /// let mut ph = ParseHelper::new_char_oriented("ab cd");
    ///
    /// let (_, all_accepted) = ph.slice_accepted(|ph| {
    ///     ph.accept_until_whitespace();
    ///     ph.accept_one_or_more_whitespace();
    ///     ph.accept_char('c');
    /// });
    ///
    /// assert_eq!(all_accepted, "ab c");
    /// ```
    pub fn slice_accepted_option(
        &mut self,
        closure: impl FnOnce(&mut Self) -> Option<()>,
    ) -> Option<&'a <T as Index<Range<usize>>>::Output>
    where
        T: Index<Range<usize>>,
    {
        let old = self.create_backup();
        let (accepted, slice) = self.slice_accepted(closure);

        match accepted {
            Some(_) => Some(slice),
            None => {
                self.restore_backup(old);
                None
            }
        }
    }
}

impl<'a, T: ?Sized, B> ParseHelper<'a, T, B>
where
    T: AsRef<[u8]>,
{
    /// Returns `true` if the end of the input has been reached.
    pub fn done(&self) -> bool {
        self.bytes_left() == 0
    }

    /// Returns how many bytes have been accepted sofar.
    /// This is equivalent to getting the current "byte position", the counter that internally
    /// keeps track of accepts
    pub fn bytes_accepted(&self) -> usize {
        self.byte_position
    }

    /// Returns how many bytes are left to parse
    pub fn bytes_left(&self) -> usize {
        self.as_ref().len() - self.byte_position
    }

    /// returns the next byte that is going to be parsed.
    pub fn upcoming_byte(&self) -> Option<u8> {
        self.input.as_ref().get(self.byte_position).copied()
    }

    /// Helper method to delegate utf8 oriented operations to byte oriented operations.
    ///
    /// # Safety
    ///
    /// The operation that is performed must ensure the parse helper ends up on a new utf8
    /// boundary.
    pub unsafe fn as_byte_oriented(&self) -> &ParseHelper<'a, T, Byte> {
        // Safety: the type is the same except for a zst
        mem::transmute(self)
    }

    /// Helper method to delegate utf8 oriented operations to byte oriented operations.
    ///
    /// # Safety
    ///
    /// The operation that is performed must ensure the parse helper ends up on a new utf8
    /// boundary.
    pub unsafe fn as_byte_oriented_mut(&mut self) -> &mut ParseHelper<'a, T, Byte> {
        // Safety: the type is the same except for a zst
        mem::transmute(self)
    }

    /// Turn this utf8 oriented parse helper into a byte oriented parse helper.
    pub fn into_byte_oriented(self) -> ParseHelper<'a, T, Byte> {
        // Safety: only a zst actually changes
        unsafe { mem::transmute_copy(&self) }
    }
}

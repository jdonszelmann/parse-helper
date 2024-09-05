#![doc=include_str!("../README.md")]
#![cfg_attr(not(feature="std"), no_std)]
#![deny(missing_docs)]

#[cfg(feature="alloc")]
extern crate alloc;


#[cfg(feature="icu")]
pub use icu_properties;

use std::{marker::PhantomData, ops::Deref};

// operations only valid on boundary::Char parse helpers
mod string;

// operations only valid on boundary::Byte parse helpers
mod byte;

// operations valid on any parse helper
mod any;

// operations to construct a parse helper
mod new;

// ZSTs marking the boundary assumption
mod boundary;

// commonly parsed tokens
mod common;

pub use boundary::{Byte, Char};

/// A wrapper around a bytes-like or string-like object that allows you to extract parts of it,
/// maybe to help implement a parser.
///
/// If you want to try out parsing something and then return to where you before, 
/// simply `.clone()` a parser. However, if you want to make your intent clearer, 
/// you might like [`backup`] and [`restore`]
///
/// [`backup`]: ParseHelper::create_backup
/// [`restore`]: ParseHelper::restore_backup
/// 
/// # Terminology
///
/// Some functions in the [`ParseHelper`] have names that mean specific things in the context of
/// parsing. This is a little overview of what different keywords mean:
///
/// * `accept*` means that the functions will take some type, match it against what's up for
///   parsing, and returns some information about what it accepted, or didn't end up accepting
/// * `accept_all` means that the function will try to accept the same thing as often as possible,
///   until the first failure. It will return whether it accepted at least once.
/// * `skip*` means not to care about what's being accepted and advance anyway
///
/// * `*with` means that the function takes some closure that determines whether the operation
///   should succeed or fail. The closure gets to inspect the input.
/// * `*upcoming` always refers to things that have not yet been accepted, but doesn't actually
///    accepts that thing. It just lets you look ahead.
///   `upcoming_byte` returns the next byte that would be accepted.
/// * `*until*` performs some operations until a certain condition becomes true for the first time.
///   The part of the input that was accepted until the where the condition evaluates to true,
///   but never includes the element on which the condition evaluates to true.
///
/// Methods always encode what type of data they work on (like, `accept_byte` vs `accept_char`),
/// except when the api works on both (like `accept` working on any bytes-like).
///
/// Any function which uses `byte` in its name accepts a single byte, 
/// while functions with `char` in its name accept a utf8 code point. 
/// Other char encodings are not supported, and if they ever will be they will be explicitly
/// named by their encoding (and not named `char`).
///
/// # Boundary assumptions
///
/// Some methods depend on the boundary assumption; there are byte and utf8 oriented parse helpers. 
/// A utf8 oriented parse helper can never have an offset that isn't on a utf8 boundary, while a
/// byte oriented parse helper can have that.
pub struct ParseHelper<'a, T: ?Sized, B> {
    input: &'a T,
    byte_position: usize,
    boundary_assumption: PhantomData<B>,
}

impl<'a, T: ?Sized, B> Clone for ParseHelper<'a, T, B> {
    fn clone(&self) -> Self {
        Self { input: self.input, byte_position: self.byte_position, boundary_assumption: PhantomData }
    }
}

// parse helpers deref to their wrapped type
impl<'a, T: ?Sized, B> Deref for ParseHelper<'a, T, B> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.input
    }
}


#[cfg(test)]
mod tests {
    use crate::ParseHelper;

    #[test]
    fn test_bytes_left() {
        let mut x = ParseHelper::new_byte_oriented("hello");
        assert_eq!(x.byte_position, 0);
        assert_eq!(x.len(), 5);
        assert_eq!(x.bytes_left(), 5);
        assert!(!x.done());
        assert!(!x.is_empty());

        x.skip_byte();
        x.skip_byte();
        x.skip_byte();

        assert_eq!(x.byte_position, 3);
        assert_eq!(x.len(), 5);
        assert_eq!(x.bytes_left(), 2);
        assert!(!x.done());
        assert!(!x.is_empty());

        x.skip_byte();
        x.skip_byte();

        assert_eq!(x.byte_position, 5);
        assert_eq!(x.len(), 5);
        assert_eq!(x.bytes_left(), 0);
        assert!(x.done());
        assert!(!x.is_empty());
    }

    #[test]
    #[should_panic="end of input reached"]
    fn test_skip_past_end() {
        let mut x = ParseHelper::new_byte_oriented("hello");
        x.skip_byte();
        x.skip_byte();
        x.skip_byte();
        x.skip_byte();
        x.skip_byte();
        x.skip_byte();
    }

    #[test]
    #[should_panic="end of input reached"]
    fn test_skip_past_end_empty() {
        let mut x = ParseHelper::new_byte_oriented("");
        x.skip_byte();
    }

    #[test]
    fn test_backup() {
        let mut x = ParseHelper::new_char_oriented("hello");

        let b = x.create_backup();

        assert_eq!(x.accept_until_char('l'), "he");
        assert_eq!(x.leftover(), "llo");

        x.restore_backup(b);

        assert_eq!(x.accept_until_char('l'), "he");
        assert_eq!(x.leftover(), "llo");
    }

    #[test]
    fn test_slice() {
        let mut x = ParseHelper::new_char_oriented("hello");

        let start = x.mark();
        assert_eq!(x.accept_until_char('l'), "he");
        assert_eq!(x.leftover(), "llo");

        let cur = x.mark();
        assert_eq!(x.slice(..cur), "he");
        assert_eq!(x.slice(cur..), "llo");
        assert_eq!(x.slice(start..cur), "he");
        assert_eq!(x.slice(..), "hello");
    }

    #[test]
    fn test_slice_byte() {
        let mut x = ParseHelper::new_byte_oriented("hello");

        let start = x.mark();
        assert_eq!(x.accept_until_byte(b'l'), b"he");
        assert_eq!(x.leftover(), b"llo");

        let cur = x.mark();
        assert_eq!(x.slice(..cur), "he");
        assert_eq!(x.slice(..=cur), "hel");
        assert_eq!(x.slice(cur..), "llo");
        assert_eq!(x.slice(start..cur), "he");
        assert_eq!(x.slice(..), "hello");
        assert_eq!(x.slice(start..=cur), "hel");
    }
}

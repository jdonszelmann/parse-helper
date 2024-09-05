use std::marker::PhantomData;

use crate::{Byte, Char, ParseHelper};

impl<'a, T: ?Sized> From<&'a T> for ParseHelper<'a, T, Char>
where
    T: AsRef<str>,
{
    fn from(value: &'a T) -> Self {
        Self::new_char_oriented(value)
    }
}

impl<'a, T: ?Sized> ParseHelper<'a, T, Byte> {
    /// Creates a new [`ParseHelper`] that assumes
    /// steps can be taken one byte at a time.
    pub fn new_byte_oriented(input: &'a T) -> Self {
        Self {
            input,
            byte_position: 0,
            boundary_assumption: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> ParseHelper<'a, T, Char>
where
    T: AsRef<str>,
{
    /// Creates a new [`ParseHelper`] that assumes
    /// steps can only be taken one utf8 codepoint at a time,
    /// and we can never end up between codepoints
    pub fn new_char_oriented(input: &'a T) -> Self {
        Self {
            input,
            byte_position: 0,
            boundary_assumption: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseHelper;

    #[test]
    fn test_from() {
        assert_eq!(ParseHelper::from("hello").leftover(), "hello");
    }
}

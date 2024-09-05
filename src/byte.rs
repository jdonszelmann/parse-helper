use core::mem;

use crate::{Byte, Char, ParseHelper};

impl<'a, T: ?Sized> ParseHelper<'a, T, Byte>
where
    T: AsRef<[u8]>,
{
    /// discard the upcoming byte
    pub fn skip_byte(&mut self) {
        self.skip_bytes(1);
    }

    /// discard the upcoming `n` byte
    pub fn skip_bytes(&mut self, n: usize) {
        assert!(n <= self.bytes_left(), "end of input reached");
        self.byte_position += n;
    }
}

// code to check whether we're on utf8 boundaries,
// only available when the underlying buffer is a string-like
impl<'a, T: ?Sized> ParseHelper<'a, T, Byte>
where
    T: AsRef<str> + AsRef<[u8]>,
{
    /// checks if the parser is currently on a utf8 boundary
    pub fn is_at_utf8_boundary(&self) -> bool {
        AsRef::<str>::as_ref(self.input).is_char_boundary(self.byte_position)
    }

    /// skips until a utf8 boundary is reached (which is never more than 4 bytes)
    pub fn skip_to_next_utf8_char_boundary(&mut self) {
        while !self.is_at_utf8_boundary() {
            self.skip_byte();
        }
    }

    /// Turn this byte oriented parse helper into a char-oriented parser,
    /// if it is currently at a utf8 boundary
    /// (see [`skip_into_char_oriented`](Self::skip_into_char_oriented)).
    ///
    /// If not, returns `None`
    pub fn into_char_oriented(self) -> Option<ParseHelper<'a, T, Char>> {
        if !self.is_at_utf8_boundary() {
            None
        } else {
            // Safety: only the zst changes
            Some(unsafe { mem::transmute_copy(&self) })
        }
    }

    /// Turn this parse helper that does not assume utf8 boundaries into one that does.
    ///
    /// If the parse helper is not currently at a utf8 boundary, it skips to the next boundary.
    pub fn skip_into_char_oriented(mut self) -> ParseHelper<'a, T, Char> {
        self.skip_to_next_utf8_char_boundary();

        // Safety: we just skipped to the next boundary so we must be at one right now.
        // into_asume_utf8_boundary returns None only when we're not at a boundary.
        unsafe { self.into_char_oriented().unwrap_unchecked() }
    }
}

impl<'a, T: ?Sized> ParseHelper<'a, T, Byte>
where
    T: AsRef<[u8]>,
{
    /// accepts a single byte from the input
    pub fn leftover(&self) -> &'a [u8] {
        &self.input.as_ref()[self.byte_position..]
    }

    /// accepts a single byte from the input
    pub fn accept_byte(&mut self, c: u8) -> bool {
        self.accept_byte_with(|x| c == x).is_some()
    }

    /// accepts a sequence of bytes-like values from the input
    ///
    /// Returns a string slice containing the same things that were asked to be accepted,
    /// but notably the lifetime is different. The new lifetime is that of the input.
    ///
    /// ```rust
    /// use parse_helper::ParseHelper;
    /// use std::borrow::Cow;
    ///
    /// let mut ph = ParseHelper::new_byte_oriented("abcdefghijklmnopqrstuvwxyz");
    ///
    /// assert_eq!(ph.accept("abc"), Some(b"abc".as_slice()));
    /// assert_eq!(ph.accept(String::from("def")), Some(b"def".as_slice()));
    /// assert_eq!(ph.accept(String::from("ghij").drain(..)), Some(b"ghij".as_slice()));
    /// assert_eq!(ph.accept(b"klm"), Some(b"klm".as_slice()));
    /// assert_eq!(ph.accept(Cow::Borrowed(b"nop".as_slice())), Some(b"nop".as_slice()));
    /// ```
    pub fn accept(&mut self, bytes: impl AsRef<[u8]>) -> Option<&'a [u8]> {
        let bytes = bytes.as_ref();
        if bytes.len() > self.bytes_left() {
            return None;
        }

        let equivalent_input =
            &self.input.as_ref()[self.byte_position..self.byte_position + bytes.len()];

        if bytes == equivalent_input {
            self.byte_position += bytes.len();
            Some(equivalent_input)
        } else {
            None
        }
    }

    /// Accepts until the closure matches the current byte.
    ///
    /// Returns what's accepted until then, but not including the matching character.
    pub fn accept_until_byte_with(&mut self, f: impl Fn(u8) -> bool) -> &'a [u8] {
        let start = self.byte_position;

        // while it doesn't match...
        while let Some(next_byte) = self.upcoming_byte() {
            if f(next_byte) {
                break;
            }

            self.byte_position += 1;
        }

        let end = self.byte_position;

        // * `begin` must not exceed `end`. (we accepted 0 or more characters)
        // * `begin` and `end` must be byte positions within the slice. (byte_position
        //   starts at 0 and never goes out of bounds, `upcoming_char` tests this)
        unsafe { self.input.as_ref().get_unchecked(start..end) }
    }

    /// Accepts until a specific character is encountered
    ///
    /// Returns what's accepted until then, but not including the matching character.
    pub fn accept_until_byte(&mut self, c: u8) -> &'a [u8] {
        self.accept_until_byte_with(|x| x == c)
    }

    /// Accepts a byte if the passed closure evaluates to true.
    ///
    /// Returns what it accepted, if anything
    pub fn accept_byte_with(&mut self, f: impl Fn(u8) -> bool) -> Option<u8> {
        let b = self.upcoming_byte()?;
        if f(b) {
            self.skip_byte();

            Some(b)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseHelper;

    #[test]
    fn accept_until_byte() {
        assert_eq!(
            ParseHelper::new_char_oriented("abc").accept_until_char('a'),
            ""
        );
        assert_eq!(
            ParseHelper::new_char_oriented("abc").accept_until_char('b'),
            "a"
        );
        assert_eq!(
            ParseHelper::new_char_oriented("abc").accept_until_char('x'),
            "abc"
        );

        let mut ph = ParseHelper::new_char_oriented("abc");
        assert_eq!(ph.accept_until_char('b'), "a");
        assert_eq!(ph.leftover(), "bc");
        assert!(ph.accept_char('b').is_some());
        assert!(ph.accept_char('c').is_some());
    }
}

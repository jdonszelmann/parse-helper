use core::str;

use crate::{Char, ParseHelper};

impl<'a, T: ?Sized> ParseHelper<'a, T, Char>
where
    T: AsRef<str> + AsRef<[u8]>,
{
    /// Returns the remaining string, the part that has not yet been accepted
    pub fn leftover(&self) -> &'a str {
        &AsRef::<str>::as_ref(self.input)[self.byte_position..]
    }

    /// returns the next character to be accepted
    pub fn upcoming_char(&self) -> Option<char> {
        // no chars left
        if self.leftover().is_empty() {
            return None;
        }

        // Safety: must be one char left, since the input was a valid string and we weren't at the end
        Some(unsafe { self.leftover().chars().next().unwrap_unchecked() })
    }

    /// Accepts a sequence of string-like values from the input.
    ///
    /// Returns a string slice containing the same things that were asked to be accepted,
    /// but notably the lifetime is different. The new lifetime is that of the input.
    ///
    /// ```rust
    /// use parse_helper::ParseHelper;
    ///
    /// let mut ph = ParseHelper::new_char_oriented("abcdefghijklmnopqrstuvwxyz");
    ///
    /// assert_eq!(ph.accept("abc"), Some("abc"));
    /// assert_eq!(ph.accept(String::from("def")), Some("def"));
    /// assert_eq!(ph.accept(String::from("ghij").drain(..)), Some("ghij"));
    /// ```
    ///
    pub fn accept(&mut self, str: impl AsRef<str>) -> Option<&'a str> {
        // Safety: bytes contains utf8 encoded characters, so after accepting it we
        // must have accepted a number of complete utf8 codepoints making us end up
        // at another boundary.
        unsafe { self.as_byte_oriented_mut().accept(str.as_ref().as_bytes()) }
            // Safety: what we get back is the exact sequence of bytes we accepted,
            // which we know is equal to some utf8 encoded string so this is valid
            .map(|x| unsafe { str::from_utf8_unchecked(x) })
    }

    /// Accepts a byte if the passed closure evaluates to true.
    ///
    /// Returns what it accepted, if anything.
    ///
    /// ```rust
    /// use parse_helper::ParseHelper;
    ///
    /// let mut ph = ParseHelper::new_char_oriented("abc");
    /// assert_eq!(ph.accept_char_with(|x| x.is_digit(10)), None);
    /// assert_eq!(ph.accept_char_with(|x| x.is_lowercase()), Some("a"));
    /// assert_eq!(ph.accept_char_with(|x| x.is_lowercase()), Some("b"));
    /// assert_eq!(ph.accept_char_with(|x| x.is_lowercase()), Some("c"));
    ///
    /// // Always None at the end
    /// assert_eq!(ph.accept_char_with(|x| true), None);
    /// ```
    pub fn accept_char_with(&mut self, f: impl Fn(char) -> bool) -> Option<&'a str> {
        let next_char = self.upcoming_char()?;
        if f(next_char) {
            let old_pos = self.byte_position;
            self.byte_position += next_char.len_utf8();
            Some(unsafe {
                AsRef::<str>::as_ref(self.input).get_unchecked(old_pos..self.byte_position)
            })
        } else {
            None
        }
    }

    /// Accepts until the closure matches the current character.
    ///
    /// Returns what's accepted until then, but not including the matching character.
    pub fn accept_until_char_with(&mut self, f: impl Fn(char) -> bool) -> &'a str {
        let start = self.byte_position;

        // while it doesn't match...
        while let Some(next_char) = self.upcoming_char() {
            if f(next_char) {
                break;
            }

            self.byte_position += next_char.len_utf8();
        }

        let end = self.byte_position;

        // * `begin` must not exceed `end`. (we accepted 0 or more characters)
        // * `begin` and `end` must be byte positions within the string slice. (byte_position
        //   starts at 0 and never goes out of bounds, `upcoming_char` tests this)
        // * `begin` and `end` must lie on UTF-8 sequence boundaries. Here we can assume
        //   all aour offsets are on utf8 boundaries because of the boundary assumption generic
        //   paramter
        unsafe { AsRef::<str>::as_ref(self.input).get_unchecked(start..end) }
    }

    /// accepts a single char from the input. Assumes the encoding is utf8.
    ///
    /// ```rust
    /// use parse_helper::ParseHelper;
    ///
    /// let mut ph = ParseHelper::new_char_oriented("abc");
    ///
    /// assert_eq!(ph.accept_char('a'), Some("a"));
    /// assert_eq!(ph.accept_char('b'), Some("b"));
    /// assert_eq!(ph.accept_char('d'), None);
    /// assert_eq!(ph.accept_char('c'), Some("c"));
    /// assert_eq!(ph.accept_char('d'), None);
    /// ```
    pub fn accept_char(&mut self, c: char) -> Option<&'a str> {
        self.accept_char_with(|x| x == c)
    }

    /// Accepts until a specific character is encountered
    ///
    /// Returns what's accepted until then, but not including the matching character.
    pub fn accept_until_char(&mut self, c: char) -> &'a str {
        self.accept_until_char_with(|x| x == c)
    }

    /// Accepts until whitespace is encountered
    ///
    /// Returns what's accepted until then, but not including the whitespace
    pub fn accept_until_whitespace(&mut self) -> &'a str {
        self.accept_until_char_with(|x| x.is_whitespace())
    }

    /// Accepts a single whitespace character.
    pub fn accept_whitespace(&mut self) -> Option<&'a str> {
        self.accept_char_with(|i| i.is_whitespace())
    }

    /// Accepts a sequence of zero or more whitespace characters.
    pub fn accept_zero_or_more_whitespace(&mut self) -> &'a str {
        self.accept_until_char_with(|x| !x.is_whitespace())
    }

    /// Accepts a sequence of one or more whitespace characters.
    pub fn accept_one_or_more_whitespace(&mut self) -> Option<&'a str> {
        if !self.upcoming_char()?.is_whitespace() {
            return None;
        }

        Some(self.accept_until_char_with(|x| !x.is_whitespace()))
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseHelper;

    #[test]
    fn accept_until_char() {
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

    #[test]
    fn accept_whitespace() {
        let mut ph = ParseHelper::new_char_oriented("ab \t   cd");
        assert_eq!(ph.accept_until_whitespace(), "ab");
        assert_eq!(ph.accept_whitespace(), Some(" "));
        assert_eq!(ph.accept_whitespace(), Some("\t"));
        assert_eq!(ph.accept_one_or_more_whitespace(), Some("   "));
        assert_eq!(ph.accept_one_or_more_whitespace(), None);
        assert_eq!(ph.accept_zero_or_more_whitespace(), "");
        assert_eq!(ph.leftover(), "cd");
    }
}

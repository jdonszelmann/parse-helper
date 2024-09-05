use core::ops::{Index, Range};

use crate::{Char, ParseHelper};

// pub struct AcceptedInt<'a> {
//     pub bytes: &'a str,
// }
//
// impl AcceptedInt {
//     pub fn get_u64(&self) -> Option<u64> {
//         self.bytes.parse()
//     }
// }
//
// pub struct AcceptedSignedInt<'a> {
//     pub bytes: &'a str,
//     pub negative: bool,
// }
//
// pub struct AcceptedString<'a> {
//     pub parsed: &'a str,
// }

impl<'a, T: ?Sized> ParseHelper<'a, T, Char>
where
    T: AsRef<str> + AsRef<[u8]>,
{
    /// Parses a rust-style identifier starting with `XID_START` and followed by zero or more
    /// `XID_CONT` characters.
    ///
    /// ```rust
    /// use parse_helper::ParseHelper;
    ///
    /// let mut ph = ParseHelper::new_char_oriented("hello wor1d 12a");
    ///
    /// assert_eq!(ph.accept_rust_ident(), Some("hello"));
    /// ph.accept_zero_or_more_whitespace();
    /// assert_eq!(ph.accept_rust_ident(), Some("wor1d"));
    /// ph.accept_zero_or_more_whitespace();
    /// assert_eq!(ph.accept_rust_ident(), None);
    ///
    /// ```
    #[cfg(feature = "icu")]
    pub fn accept_rust_ident(&mut self) -> Option<&'a <T as Index<Range<usize>>>::Output>
    where
        T: Index<Range<usize>>,
    {
        let start = icu_properties::sets::xid_start();
        let cont = icu_properties::sets::xid_continue();

        self.slice_accepted_option(|ph| {
            ph.accept_char_with(|x| start.contains(x))?;
            while ph.accept_char_with(|x| cont.contains(x)).is_some() {}

            Some(())
        })
    }
    //
    // /// Parse an integer (excluding any possible minus sign)
    // ///
    // ///
    // pub fn accept_int(&self) -> Option<AcceptedInt> {
    //     todo!()
    // }
    //
    // pub fn signed_int(&self) -> Option<&'a str> {
    //     todo!()
    // }
    //
    // pub fn single_quoted_string(&self) -> Option<&'a str> {
    //     todo!()
    // }
    //
    // pub fn double_quoted_string(&self) -> Option<&'a str> {
    //     todo!()
    // }
    //
    // /// Parses a string like rust would, including escape sequences.
    // pub fn rust_string(&self) -> Option<&'a str> {
    //     todo!()
    // }
}

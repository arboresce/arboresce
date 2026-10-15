//! Private JSON mechanics for the fixed reference/locator record graph only.

use std::{error::Error, fmt, fmt::Write};

use crate::references::UnsignedJsonInteger;

pub const MAX_LOCATOR_WIRE_BYTES: usize = 1_048_576;
/// A conservative bound for every complete checked locator/reference output.
pub const MAX_LOCATOR_WIRE_OUTPUT_BYTES: usize = 32_768;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidLocatorWire;

impl fmt::Display for InvalidLocatorWire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid locator wire value")
    }
}
impl Error for InvalidLocatorWire {}

pub(crate) fn parse<'a, T>(
    input: &'a [u8],
    read: impl FnOnce(&mut Reader<'a>) -> Result<T, InvalidLocatorWire>,
) -> Result<T, InvalidLocatorWire> {
    if input.len() > MAX_LOCATOR_WIRE_BYTES {
        return Err(InvalidLocatorWire);
    }
    let mut reader = Reader {
        input: std::str::from_utf8(input).map_err(|_| InvalidLocatorWire)?,
        position: 0,
    };
    let value = read(&mut reader)?;
    reader.whitespace();
    if reader.position != reader.input.len() {
        return Err(InvalidLocatorWire);
    }
    Ok(value)
}

/// The only callers describe acyclic records of at most five containers. A union
/// recognizes nineteen possible names but may consume at most ten members.
pub(crate) struct Reader<'a> {
    input: &'a str,
    position: usize,
}

impl Reader<'_> {
    fn peek(&self) -> Option<char> {
        self.input.get(self.position..)?.chars().next()
    }

    fn take(&mut self) -> Result<char, InvalidLocatorWire> {
        let value = self.peek().ok_or(InvalidLocatorWire)?;
        // A complete char in the bounded input cannot advance past its end.
        self.position += value.len_utf8();
        Ok(value)
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\r' | '\n')) {
            self.position += 1;
        }
    }

    fn token(&mut self, expected: char) -> Result<(), InvalidLocatorWire> {
        self.whitespace();
        if self.take()? == expected {
            Ok(())
        } else {
            Err(InvalidLocatorWire)
        }
    }

    fn hex4(&mut self) -> Result<u16, InvalidLocatorWire> {
        let mut value = 0u16;
        for _ in 0..4 {
            let digit = match self.take()? {
                c @ '0'..='9' => c as u16 - u16::from(b'0'),
                c @ 'a'..='f' => c as u16 - u16::from(b'a') + 10,
                c @ 'A'..='F' => c as u16 - u16::from(b'A') + 10,
                _ => return Err(InvalidLocatorWire),
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn escaped(&mut self) -> Result<char, InvalidLocatorWire> {
        Ok(match self.take()? {
            '"' => '"',
            '\\' => '\\',
            '/' => '/',
            'b' => '\u{8}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => {
                let first = self.hex4()?;
                let code = if (0xd800..=0xdbff).contains(&first) {
                    if self.take()? != '\\' || self.take()? != 'u' {
                        return Err(InvalidLocatorWire);
                    }
                    let second = self.hex4()?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(InvalidLocatorWire);
                    }
                    0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(second) - 0xdc00
                } else {
                    u32::from(first)
                };
                char::from_u32(code).ok_or(InvalidLocatorWire)?
            }
            _ => return Err(InvalidLocatorWire),
        })
    }

    pub(crate) fn string(&mut self, maximum: usize) -> Result<String, InvalidLocatorWire> {
        self.token('"')?;
        let mut result = String::with_capacity(maximum);
        let mut scalars = 0;
        loop {
            let value = match self.take()? {
                '"' => return Ok(result),
                '\\' => self.escaped()?,
                c if c <= '\u{1f}' => return Err(InvalidLocatorWire),
                c => c,
            };
            if scalars == maximum {
                return Err(InvalidLocatorWire);
            }
            result.push(value);
            scalars += 1;
        }
    }

    pub(crate) fn unsigned(&mut self) -> Result<u64, InvalidLocatorWire> {
        self.whitespace();
        let start = self.position;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.position += 1;
        }
        let token = self
            .input
            .get(start..self.position)
            .ok_or(InvalidLocatorWire)?;
        UnsignedJsonInteger::from_token(token.as_bytes()).map(UnsignedJsonInteger::value)
    }

    pub(crate) fn object(
        &mut self,
        names: &[&str],
        mut field: impl FnMut(&mut Self, &str) -> Result<(), InvalidLocatorWire>,
    ) -> Result<(), InvalidLocatorWire> {
        self.token('{')?;
        let mut seen = 0u32;
        self.whitespace();
        if self.peek() == Some('}') {
            self.take()?;
            return Ok(());
        }
        loop {
            if seen.count_ones() == 10 {
                return Err(InvalidLocatorWire);
            }
            let key = self.string(21)?;
            self.token(':')?;
            let index = names
                .iter()
                .position(|name| *name == key)
                .ok_or(InvalidLocatorWire)?;
            // All private name sets contain at most nineteen entries.
            let bit = 1u32 << index;
            if seen & bit != 0 {
                return Err(InvalidLocatorWire);
            }
            seen |= bit;
            field(self, &key)?;
            self.whitespace();
            match self.take()? {
                '}' => return Ok(()),
                ',' => {}
                _ => return Err(InvalidLocatorWire),
            }
        }
    }
}

pub(crate) fn quoted(f: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
    f.write_str("\"")?;
    for c in input.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\u{8}' => f.write_str("\\b")?,
            '\u{c}' => f.write_str("\\f")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            c if c <= '\u{1f}' => write!(f, "\\u{:04x}", u32::from(c))?,
            c => f.write_char(c)?,
        }
    }
    f.write_str("\"")
}

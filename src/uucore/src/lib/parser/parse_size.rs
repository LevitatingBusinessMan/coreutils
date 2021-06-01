//  * This file is part of the uutils coreutils package.
//  *
//  * For the full copyright and license information, please view the LICENSE
//  * file that was distributed with this source code.

use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

/// Parse a size string into a number of bytes.
///
/// A size string comprises an integer and an optional unit. The unit
/// may be K, M, G, T, P, E, Z or Y (powers of 1024), or KB, MB,
/// etc. (powers of 1000), or b which is 512.
/// Binary prefixes can be used, too: KiB=K, MiB=M, and so on.
///
/// # Errors
///
/// Will return `ParseSizeError` if it’s not possible to parse this
/// string into a number, e.g. if the string does not begin with a
/// numeral, or if the unit is not one of the supported units described
/// in the preceding section.
///
/// # Examples
///
/// ```rust
/// use uucore::parse_size::parse_size;
/// assert_eq!(Ok(123), parse_size("123"));
/// assert_eq!(Ok(9 * 1000), parse_size("9kB")); // kB is 1000
/// assert_eq!(Ok(2 * 1024), parse_size("2K")); // K is 1024
/// ```
pub fn parse_size(size: &str) -> Result<usize, ParseSizeError> {
    if size.is_empty() {
        return Err(ParseSizeError::parse_failure(size));
    }
    // Get the numeric part of the size argument. For example, if the
    // argument is "123K", then the numeric part is "123".
    let numeric_string: String = size.chars().take_while(|c| c.is_digit(10)).collect();
    let number: usize = if !numeric_string.is_empty() {
        match numeric_string.parse() {
            Ok(n) => n,
            Err(_) => return Err(ParseSizeError::parse_failure(size)),
        }
    } else {
        1
    };

    // Get the alphabetic units part of the size argument and compute
    // the factor it represents. For example, if the argument is "123K",
    // then the unit part is "K" and the factor is 1024. This may be the
    // empty string, in which case, the factor is 1.
    let unit = &size[numeric_string.len()..];
    let (base, exponent): (u128, u32) = match unit {
        "" => (1, 0),
        "b" => (512, 1), // (`head` and `tail` use "b")
        "KiB" | "kiB" | "K" | "k" => (1024, 1),
        "MiB" | "miB" | "M" | "m" => (1024, 2),
        "GiB" | "giB" | "G" | "g" => (1024, 3),
        "TiB" | "tiB" | "T" | "t" => (1024, 4),
        "PiB" | "piB" | "P" | "p" => (1024, 5),
        "EiB" | "eiB" | "E" | "e" => (1024, 6),
        "ZiB" | "ziB" | "Z" | "z" => (1024, 7),
        "YiB" | "yiB" | "Y" | "y" => (1024, 8),
        "KB" | "kB" => (1000, 1),
        "MB" | "mB" => (1000, 2),
        "GB" | "gB" => (1000, 3),
        "TB" | "tB" => (1000, 4),
        "PB" | "pB" => (1000, 5),
        "EB" | "eB" => (1000, 6),
        "ZB" | "zB" => (1000, 7),
        "YB" | "yB" => (1000, 8),
        _ => return Err(ParseSizeError::parse_failure(size)),
    };
    let factor = match usize::try_from(base.pow(exponent)) {
        Ok(n) => n,
        Err(_) => return Err(ParseSizeError::size_too_big(size)),
    };
    match number.checked_mul(factor) {
        Some(n) => Ok(n),
        None => Err(ParseSizeError::size_too_big(size)),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseSizeError {
    ParseFailure(String), // Syntax
    SizeTooBig(String),   // Overflow
}

impl Error for ParseSizeError {
    fn description(&self) -> &str {
        match *self {
            ParseSizeError::ParseFailure(ref s) => &*s,
            ParseSizeError::SizeTooBig(ref s) => &*s,
        }
    }
}

impl fmt::Display for ParseSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ParseSizeError::ParseFailure(s) => s,
            ParseSizeError::SizeTooBig(s) => s,
        };
        write!(f, "{}", s)
    }
}

impl ParseSizeError {
    fn parse_failure(s: &str) -> ParseSizeError {
        // has to be handled in the respective uutils because strings differ, e.g.
        // truncate: Invalid number: ‘fb’
        // tail: invalid number of bytes: ‘fb’
        ParseSizeError::ParseFailure(format!("‘{}’", s))
    }

    fn size_too_big(s: &str) -> ParseSizeError {
        // has to be handled in the respective uutils because strings differ, e.g.
        // truncate: Invalid number: ‘1Y’: Value too large to be stored in data type
        // tail: invalid number of bytes: ‘1Y’: Value too large to be stored in data type
        ParseSizeError::SizeTooBig(format!(
            "‘{}’: Value too large to be stored in data type",
            s
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant_eq(a: &ParseSizeError, b: &ParseSizeError) -> bool {
        std::mem::discriminant(a) == std::mem::discriminant(b)
    }

    #[test]
    fn all_suffixes() {
        // Units  are  K,M,G,T,P,E,Z,Y  (powers  of 1024) or KB,MB,... (powers of 1000).
        // Binary prefixes can be used, too: KiB=K, MiB=M, and so on.
        let suffixes = [
            ('K', 1u32),
            ('M', 2u32),
            ('G', 3u32),
            ('T', 4u32),
            ('P', 5u32),
            ('E', 6u32),
            #[cfg(target_pointer_width = "128")]
            ('Z', 7u32), // ParseSizeError::SizeTooBig on x64
            #[cfg(target_pointer_width = "128")]
            ('Y', 8u32), // ParseSizeError::SizeTooBig on x64
        ];

        for &(c, exp) in &suffixes {
            let s = format!("2{}B", c); // KB
            assert_eq!(Ok((2 * (1000_u128).pow(exp)) as usize), parse_size(&s));
            let s = format!("2{}", c); // K
            assert_eq!(Ok((2 * (1024_u128).pow(exp)) as usize), parse_size(&s));
            let s = format!("2{}iB", c); // KiB
            assert_eq!(Ok((2 * (1024_u128).pow(exp)) as usize), parse_size(&s));
            let s = format!("2{}iB", c.to_lowercase()); // kiB
            assert_eq!(Ok((2 * (1024_u128).pow(exp)) as usize), parse_size(&s));

            // suffix only
            let s = format!("{}B", c); // KB
            assert_eq!(Ok(((1000_u128).pow(exp)) as usize), parse_size(&s));
            let s = format!("{}", c); // K
            assert_eq!(Ok(((1024_u128).pow(exp)) as usize), parse_size(&s));
            let s = format!("{}iB", c); // KiB
            assert_eq!(Ok(((1024_u128).pow(exp)) as usize), parse_size(&s));
            let s = format!("{}iB", c.to_lowercase()); // kiB
            assert_eq!(Ok(((1024_u128).pow(exp)) as usize), parse_size(&s));
        }
    }

    #[test]
    #[cfg(not(target_pointer_width = "128"))]
    fn overflow_x64() {
        assert!(parse_size("10000000000000000000000").is_err());
        assert!(parse_size("1000000000T").is_err());
        assert!(parse_size("100000P").is_err());
        assert!(parse_size("100E").is_err());
        assert!(parse_size("1Z").is_err());
        assert!(parse_size("1Y").is_err());

        assert!(variant_eq(
            &parse_size("1Z").unwrap_err(),
            &ParseSizeError::SizeTooBig(String::new())
        ));

        assert_eq!(
            ParseSizeError::SizeTooBig(
                "‘1Y’: Value too large to be stored in data type".to_string()
            ),
            parse_size("1Y").unwrap_err()
        );
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn overflow_x32() {
        assert!(variant_eq(
            &parse_size("1T").unwrap_err(),
            &ParseSizeError::SizeTooBig(String::new())
        ));
        assert!(variant_eq(
            &parse_size("1000G").unwrap_err(),
            &ParseSizeError::SizeTooBig(String::new())
        ));
    }

    #[test]
    fn invalid_syntax() {
        let test_strings = ["328hdsf3290", "5MiB nonsense", "5mib", "biB", "-", ""];
        for &test_string in &test_strings {
            assert_eq!(
                parse_size(test_string).unwrap_err(),
                ParseSizeError::ParseFailure(format!("‘{}’", test_string))
            );
        }
    }

    #[test]
    fn b_suffix() {
        assert_eq!(Ok(3 * 512), parse_size("3b")); // b is 512
    }

    #[test]
    fn no_suffix() {
        assert_eq!(Ok(1234), parse_size("1234"));
        assert_eq!(Ok(0), parse_size("0"));
    }

    #[test]
    fn kilobytes_suffix() {
        assert_eq!(Ok(123 * 1000), parse_size("123KB")); // KB is 1000
        assert_eq!(Ok(9 * 1000), parse_size("9kB")); // kB is 1000
        assert_eq!(Ok(2 * 1024), parse_size("2K")); // K is 1024
        assert_eq!(Ok(0), parse_size("0K"));
        assert_eq!(Ok(0), parse_size("0KB"));
        assert_eq!(Ok(1000), parse_size("KB"));
        assert_eq!(Ok(1024), parse_size("K"));
    }

    #[test]
    fn megabytes_suffix() {
        assert_eq!(Ok(123 * 1024 * 1024), parse_size("123M"));
        assert_eq!(Ok(123 * 1000 * 1000), parse_size("123MB"));
        assert_eq!(Ok(1024 * 1024), parse_size("M"));
        assert_eq!(Ok(1000 * 1000), parse_size("MB"));
    }
}

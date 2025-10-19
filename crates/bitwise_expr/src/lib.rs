#![no_std]

extern crate alloc;

use alloc::string::String;
use nom::{branch::alt, bytes::complete::tag, character::complete::digit1, combinator::{map, map_res}, multi::many0, sequence::{delimited, pair, preceded}, Err, IResult};
use core::{fmt, num};
use core::fmt::Write;

struct Writer<'a> {
    buffer: &'a mut [u8],
    offset: usize
}

impl fmt::Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining_len = self.buffer.len() - self.offset;
        let len_to_copy = core::cmp::min(bytes.len(), remaining_len);
        if len_to_copy > 0 {
            self.buffer[self.offset..self.offset + len_to_copy].copy_from_slice(&bytes[..len_to_copy]);
            self.offset += len_to_copy;
        }
        if len_to_copy < bytes.len() {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

fn preprocess<'a>(input: &str, output_buff: &'a mut [u8]) -> Result<&'a str, &'static str> {
    let mut writer = Writer {
        buffer: output_buff,
        offset: 0
    };
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\r' | '\n' => continue,
            // convert hex and binary e.g. 0xAAA 0b111
            '0' => {
                if let Some(next_char) = chars.peek() {
                    match next_char {
                        'x' => {
                            chars.next();
                            let mut num_str = String::new();
                            while let Some(&ch) = chars.peek() {
                                if ch.is_ascii_hexdigit() {
                                    num_str.push(ch);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            let val = u64::from_str_radix(&num_str, 16).map_err(|_| "Invalid hex number")?;
                            write!(writer, "{}", val).map_err(|_| "Output buffer too small")?;
                        },
                        'b' => {
                            chars.next();
                            let mut num_str = String::new();
                            while let Some(&ch) = chars.peek() {
                                if ch == '0' || ch == '1' {
                                    num_str.push(ch);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            let val = u64::from_str_radix(&num_str, 2).map_err(|_| "Invalid binary number")?;
                            write!(writer, "{}", val).map_err(|_| "Output buffer too small")?;
                        },
                        _ => {
                            writer.write_char('0').map_err(|_| "Output buffer too small")?;
                        }
                    }
                } else {
                    writer.write_char('0').map_err(|_| "Output buffer too small")?;
                }
            }
            _ => {
                writer.write_char(c).map_err(|_| "Output buffer too small")?;
            }
        }
    }
    core::str::from_utf8(&writer.buffer[..writer.offset]).map_err(|_| "Invalid UTF-8 in output")
}

fn parse_number(input: &str) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

fn parse_factor(input: &str) -> IResult<&str, i64> {
    alt((
        parse_number,
        delimited(tag("("), parse_equation, tag(")")),
        map(
            preceded(tag("~"), parse_factor),
            |val| !val
        ),
    ))(input)
}

fn parse_multiplicative(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_factor(input)?;
    let (input, ops_and_vals) = many0(
        pair(
            alt((
                tag("*"),
                tag("/")
            )),
            parse_factor
        )
    )(input)?;
    for (op, val) in ops_and_vals {
        if op == "*" {
            result *= val;
        } else {
            if val == 0 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    "Division by zero", 
                    nom::error::ErrorKind::MapRes
                )));

            } else {
                result /= val;
            }
        }
    }
    Ok((input, result))
}

// alias for the current top level parse
fn parse_equation(input: &str) -> IResult<&str, i64> {
    parse_bitwise_or(input)
}

fn parse_bitwise_or(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_bitwise_xor(input)?;
    let (input, ops) = many0(
        preceded(tag("|"), parse_bitwise_xor)
    )(input)?;
    for val in ops {
        result |= val;
    }
    Ok((input, result))
}

fn parse_bitwise_xor(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_bitwise_and(input)?;
    let (input, ops) = many0(
        preceded(tag("^"), parse_bitwise_and)
    )(input)?;
    for val in ops {
        result ^= val;
    }
    Ok((input, result))
}

fn parse_bitwise_and(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_shift(input)?;
    let (input, ops) = many0(
        preceded(tag("&"), parse_shift)
    )(input)?;
    for val in ops {
        result &= val;
    }
    Ok((input, result))
}

fn parse_shift(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_additive(input)?;
    let (input, ops_and_vals) = many0(
        pair(alt((tag("<<"), tag(">>"))), parse_additive)
    )(input)?;
    for (op, val) in ops_and_vals {
        if op == "<<" {
            result = result.wrapping_shl(val as u32);
        } else {
            result = result.wrapping_shr(val as u32);
        }
    }
    Ok((input, result))
}

fn parse_additive(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_multiplicative(input)?;
    let (input, ops_and_vals) = many0(
        pair(
            alt((
                tag("+"),
                tag("-")
            )),
            parse_multiplicative
        )
    )(input)?;
    for (op, val) in ops_and_vals {
        if op == "+" {
            result += val;
        } else {
            result -= val;
        }
    }
    Ok((input, result))
}

pub fn evaluate(input: &str) -> Result<i64, &'static str> {
    let mut buffer = [0u8; 256];
    let new_input = preprocess(input, &mut buffer)?;
    use nom::Finish;
    match parse_equation(new_input).finish() {
        Ok((_remaining, value)) => Ok(value),
        Err(_) => Err("Failed to evaluate expression"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess() {
        // Each test gets its own clean buffer.
        let mut buf1 = [0u8; 128];
        assert_eq!(preprocess(" 1 + 2   *  3 ", &mut buf1), Ok("1+2*3"));

        let mut buf2 = [0u8; 128];
        assert_eq!(preprocess("0x10 + 0xA", &mut buf2), Ok("16+10"));

        let mut buf3 = [0u8; 128];
        assert_eq!(preprocess("0x10+0xA", &mut buf3), Ok("16+10")); // This will now pass

        let mut buf4 = [0u8; 128];
        assert_eq!(preprocess("0b101 << 0b11 + 0", &mut buf4), Ok("5<<3+0"));

        let mut buf5 = [0u8; 128];
        assert_eq!(preprocess("0b101 << 0xA", &mut buf5), Ok("5<<10"));

        // Test for small buffer still works correctly
        let mut small_buf = [0u8; 3];
        assert_eq!(preprocess("0b101 << 0xA", &mut small_buf), Err("Output buffer too small"));
    }

    #[test]
    fn test_parse_factor() {
        assert_eq!(parse_factor("123"), Ok(("", 123)));
        assert_eq!(parse_factor("777blahblah"), Ok(("blahblah", 777)));
    }

    #[test]
    fn test_parse_multiplicative() {
        assert_eq!(parse_multiplicative("1*2"), Ok(("", 2)));
        assert_eq!(parse_multiplicative("3*4*5"), Ok(("", 60)));
        assert_eq!(parse_multiplicative("10"), Ok(("", 10)));
        assert_eq!(parse_multiplicative("7*2+3"), Ok(("+3", 14)));
    }

    #[test]
    fn test_parse_additive() {
        assert_eq!(parse_additive("1+2"), Ok(("", 3)));
        assert_eq!(parse_additive("3+4+5"), Ok(("", 12)));
        assert_eq!(parse_additive("10"), Ok(("", 10)));
        assert_eq!(parse_additive("2*3+4"), Ok(("", 10)));
        assert_eq!(parse_additive("2+3*4"), Ok(("", 14)));
        assert_eq!(parse_additive("1+2*3+4"), Ok(("", 11)));
    }

    #[test]
    fn test_evaluate() {
        assert_eq!(evaluate(" 2 * 3 + 4 "), Ok(10));
        assert_eq!(evaluate(" 2 + 3 * 4 "), Ok(14));
        assert_eq!(evaluate(" 1 + 0x10 * 0b10 "), Ok(33));
        assert_eq!(evaluate("(99)"), Ok(99));
        assert_eq!(evaluate("5 * (2 + 3)"), Ok(25));
        assert_eq!(evaluate("(2 + 3) * 5"), Ok(25));
        assert_eq!(evaluate("((0x0002 + 3) * 4) + 5"), Ok(25));
        assert_eq!(evaluate("10 - 5"), Ok(5));
        assert_eq!(evaluate("20 / 4"), Ok(5));
        assert_eq!(evaluate("10 - 2 * 3"), Ok(4));
        assert_eq!(evaluate("20 / 2 - 3"), Ok(7));
        assert_eq!(evaluate("10 - 3 - 2"), Ok(5));
        assert_eq!(evaluate("100 / 10 / 2"), Ok(5));
        assert_eq!(evaluate("100 / (10 / 2)"), Ok(20));
        assert_eq!(evaluate("(10 - 2) * 3"), Ok(24));
        assert_eq!(evaluate("10 << 2"), Ok(40));
        assert_eq!(evaluate("10 >> 1"), Ok(5));
        assert_eq!(evaluate("0b1100 & 0b1010"), Ok(0b1000));
        assert_eq!(evaluate("0b1100 | 0b1010"), Ok(0b1110));
        assert_eq!(evaluate("0b1100 ^ 0b1010"), Ok(0b0110));
        assert_eq!(evaluate("2 * 5 << 1"), Ok(20));
        assert_eq!(evaluate("1 << 2 * 5"), Ok(1024));
        assert_eq!(evaluate("1 + 2 << 3"), Ok(24));
        assert_eq!(evaluate("0b1111 & 0b0101 << 1"), Ok(0b1111 & 0b0101 << 1));
        assert_eq!(evaluate("0b10 | 0b01 & 0b11"), Ok(0b11));
        assert_eq!(evaluate("(1 << 12) & 0x0A + 100 * 2"), Ok((1 << 12) & 0x0A + 100 * 2));
        assert_eq!(evaluate("~0"), Ok(!0));
        assert_eq!(evaluate("~1"), Ok(!1));
        assert_eq!(evaluate("~~1"), Ok(!!1));
        assert_eq!(evaluate("~1 * 5"), Ok(!1 * 5));
        assert_eq!(evaluate("~10 + 5"), Ok(!10 + 5));
        assert_eq!(evaluate("~(10 + 5)"), Ok(!(10 + 5)));
    }

    #[test]
    fn test_trouble() {
        assert_eq!(evaluate("0x01 + 2"), Ok(0x01+2));
        assert_eq!(evaluate("0xC+2"), Ok(0xC+2));
        assert_eq!(evaluate("0xC*2"), Ok(0xC*2));
    }
}
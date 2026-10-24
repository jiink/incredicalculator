#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use nom::{branch::alt, bytes::complete::tag, character::complete::digit1, combinator::{map, map_res}, multi::many0, sequence::{delimited, pair, preceded}, IResult};
use core::fmt;
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

/// The integer type expressions are evaluated as. Every literal and every
/// intermediate result is wrapped into this type, like fixed-width hardware.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntFormat {
    pub bits: u32,
    pub signed: bool,
}

impl IntFormat {
    pub const I64: IntFormat = IntFormat { bits: 64, signed: true };

    pub const fn new(bits: u32, signed: bool) -> Self {
        Self { bits, signed }
    }

    pub fn mask(self) -> u64 {
        if self.bits >= 64 {
            u64::MAX
        } else {
            (1u64 << self.bits) - 1
        }
    }

    /// Truncates `value` to `bits` bits, then sign-extends it if signed.
    pub fn wrap(self, value: i64) -> i64 {
        if self.bits >= 64 {
            return value;
        }
        let mask = self.mask();
        let raw = (value as u64) & mask;
        if self.signed && raw & (1u64 << (self.bits - 1)) != 0 {
            (raw | !mask) as i64
        } else {
            raw as i64
        }
    }
}

fn parse_number(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<u64>().map(|v| fmt.wrap(v as i64)))(input)
}

fn parse_factor(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    alt((
        |i| parse_number(i, fmt),
        delimited(tag("("), |i| parse_equation(i, fmt), tag(")")),
        map(
            preceded(tag("~"), |i| parse_factor(i, fmt)),
            |val| fmt.wrap(!val)
        ),
        map(
            preceded(tag("-"), |i| parse_factor(i, fmt)),
            |val: i64| fmt.wrap(val.wrapping_neg())
        ),
    ))(input)
}

fn parse_multiplicative(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    let (input, mut result) = parse_factor(input, fmt)?;
    let (input, ops_and_vals) = many0(
        pair(
            alt((
                tag("*"),
                tag("/"),
                tag("%")
            )),
            |i| parse_factor(i, fmt)
        )
    )(input)?;
    for (op, val) in ops_and_vals {
        match op {
            "*" => result = result.wrapping_mul(val),
            "/" => {
                if val == 0 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        "Division by zero",
                        nom::error::ErrorKind::MapRes
                    )));
                }
                result = result.wrapping_div(val);
            }
            "%" => {
                if val == 0 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        "Division by zero",
                        nom::error::ErrorKind::MapRes
                    )));
                }
                result = result.wrapping_rem(val);
            }
            _ => unreachable!("unexpected operator {op}"),
        }
        result = fmt.wrap(result);
    }
    Ok((input, result))
}

// alias for the current top level parse
fn parse_equation(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    parse_bitwise_or(input, fmt)
}

fn parse_bitwise_or(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    let (input, mut result) = parse_bitwise_xor(input, fmt)?;
    let (input, ops) = many0(
        preceded(tag("|"), |i| parse_bitwise_xor(i, fmt))
    )(input)?;
    for val in ops {
        result = fmt.wrap(result | val);
    }
    Ok((input, result))
}

fn parse_bitwise_xor(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    let (input, mut result) = parse_bitwise_and(input, fmt)?;
    let (input, ops) = many0(
        preceded(tag("^"), |i| parse_bitwise_and(i, fmt))
    )(input)?;
    for val in ops {
        result = fmt.wrap(result ^ val);
    }
    Ok((input, result))
}

fn parse_bitwise_and(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    let (input, mut result) = parse_shift(input, fmt)?;
    let (input, ops) = many0(
        preceded(tag("&"), |i| parse_shift(i, fmt))
    )(input)?;
    for val in ops {
        result = fmt.wrap(result & val);
    }
    Ok((input, result))
}

fn parse_shift(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    let (input, mut result) = parse_additive(input, fmt)?;
    let (input, ops_and_vals) = many0(
        pair(alt((tag("<<"), tag(">>"))), |i| parse_additive(i, fmt))
    )(input)?;
    for (op, val) in ops_and_vals {
        if op == "<<" {
            result = result.wrapping_shl(val as u32);
        } else {
            result = result.wrapping_shr(val as u32);
        }
        result = fmt.wrap(result);
    }
    Ok((input, result))
}

fn parse_additive(input: &str, fmt: IntFormat) -> IResult<&str, i64> {
    let (input, mut result) = parse_multiplicative(input, fmt)?;
    let (input, ops_and_vals) = many0(
        pair(
            alt((
                tag("+"),
                tag("-")
            )),
            |i| parse_multiplicative(i, fmt)
        )
    )(input)?;
    for (op, val) in ops_and_vals {
        if op == "+" {
            result = result.wrapping_add(val);
        } else {
            result = result.wrapping_sub(val);
        }
        result = fmt.wrap(result);
    }
    Ok((input, result))
}

pub fn evaluate(input: &str) -> Result<i64, &'static str> {
    evaluate_as(input, IntFormat::I64)
}

pub fn evaluate_as(input: &str, fmt: IntFormat) -> Result<i64, &'static str> {
    let mut buffer = [0u8; 256];
    let new_input = preprocess(input, &mut buffer)?;
    use nom::Finish;
    match parse_equation(new_input, fmt).finish() {
        Ok((_remaining, value)) => {
            Ok(value)
        },
        Err(_) => Err("Failed to evaluate expression"),
    }
}

pub fn evaluate_str(input: &str) -> String {
    evaluate_str_as(input, IntFormat::I64)
}

pub fn evaluate_str_as(input: &str, fmt: IntFormat) -> String {
    match self::evaluate_as(input, fmt) {
        Ok(value) => {
            let mut buf = itoa::Buffer::new();
            buf.format(value).to_string()
        },
        Err(msg) => String::from(msg)
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
        assert_eq!(parse_factor("123", IntFormat::I64), Ok(("", 123)));
        assert_eq!(parse_factor("777blahblah", IntFormat::I64), Ok(("blahblah", 777)));
    }

    #[test]
    fn test_parse_multiplicative() {
        assert_eq!(parse_multiplicative("1*2", IntFormat::I64), Ok(("", 2)));
        assert_eq!(parse_multiplicative("3*4*5", IntFormat::I64), Ok(("", 60)));
        assert_eq!(parse_multiplicative("10", IntFormat::I64), Ok(("", 10)));
        assert_eq!(parse_multiplicative("7*2+3", IntFormat::I64), Ok(("+3", 14)));
        assert_eq!(parse_multiplicative("10%3", IntFormat::I64), Ok(("", 1)));
        assert_eq!(parse_multiplicative("0%300", IntFormat::I64), Ok(("", 0)));
        assert_eq!(parse_multiplicative("300%300", IntFormat::I64), Ok(("", 0)));
    }

    #[test]
    fn test_parse_additive() {
        assert_eq!(parse_additive("1+2", IntFormat::I64), Ok(("", 3)));
        assert_eq!(parse_additive("3+4+5", IntFormat::I64), Ok(("", 12)));
        assert_eq!(parse_additive("10", IntFormat::I64), Ok(("", 10)));
        assert_eq!(parse_additive("2*3+4", IntFormat::I64), Ok(("", 10)));
        assert_eq!(parse_additive("2+3*4", IntFormat::I64), Ok(("", 14)));
        assert_eq!(parse_additive("1+2*3+4", IntFormat::I64), Ok(("", 11)));
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
        assert_eq!(evaluate("10 % 3"), Ok(1));
        assert_eq!(evaluate("12 % (5 - 2)"), Ok(0));
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

    #[test]
    fn test_negatives() {
        assert_eq!(evaluate("-5"), Ok(-5));
        assert_eq!(evaluate("-5 + 3"), Ok(-2));
        assert_eq!(evaluate("3 - -5"), Ok(8));
        assert_eq!(evaluate("3 * -5"), Ok(-15));
        assert_eq!(evaluate("-(2 + 3)"), Ok(-5));
        assert_eq!(evaluate("--5"), Ok(5));
        assert_eq!(evaluate("~-1"), Ok(0));
        assert_eq!(evaluate("-0x10"), Ok(-16));
        assert_eq!(evaluate("-1 & 0xFF"), Ok(0xFF));
    }

    #[test]
    fn test_int_formats() {
        let u8_ = IntFormat::new(8, false);
        let i8_ = IntFormat::new(8, true);
        let u16_ = IntFormat::new(16, false);
        let i32_ = IntFormat::new(32, true);
        let u32_ = IntFormat::new(32, false);
        assert_eq!(evaluate_as("200 + 100", u8_), Ok(44));
        assert_eq!(evaluate_as("-1", u8_), Ok(255));
        assert_eq!(evaluate_as("~0", u8_), Ok(255));
        assert_eq!(evaluate_as("~0 >> 1", u8_), Ok(127));
        assert_eq!(evaluate_as("~0 / 2", u8_), Ok(127));
        assert_eq!(evaluate_as("0x80 << 1", u8_), Ok(0));
        assert_eq!(evaluate_as("127 + 1", i8_), Ok(-128));
        assert_eq!(evaluate_as("0xFF", i8_), Ok(-1));
        assert_eq!(evaluate_as("0xFF >> 1", i8_), Ok(-1));
        assert_eq!(evaluate_as("0x12345", u16_), Ok(0x2345));
        assert_eq!(evaluate_as("0x80000000", i32_), Ok(i32::MIN as i64));
        assert_eq!(evaluate_as("-2147483648", u32_), Ok(0x80000000));
        assert_eq!(evaluate_as("0xFFFFFFFF * 0xFFFFFFFF", u32_), Ok(1));
    }
}
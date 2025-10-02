#![no_std]

extern crate alloc;

use alloc::string::String;
use nom::{bytes::complete::tag, character::complete::{digit1}, combinator::map_res, multi::many0, sequence::{delimited, preceded}, IResult};
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

pub fn preprocess<'a>(input: &str, output_buff: &'a mut [u8]) -> Result<&'a str, &'static str> {
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
                            let num_str: String = chars.by_ref().take_while(|ch| ch.is_ascii_hexdigit()).collect();
                            let val = u64::from_str_radix(&num_str, 16).map_err(|_| "Invalid hex number")?;
                            write!(writer, "{}", val).map_err(|_| "Output buffer too small")?;
                        },
                        'b' => {
                            chars.next();
                            let num_str: String = chars.by_ref().take_while(|ch| *ch == '0' || *ch == '1').collect();
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

fn parse_factor(input: &str) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

fn parse_term(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_factor(input)?;
    let (input, multiplications) = many0(preceded(tag("*"), parse_factor))(input)?;
    for num in multiplications {
        result *= num;
    }
    Ok((input, result))
}

fn parse_expr(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_term(input)?;
    let (input, additions) = many0(preceded(tag("+"), parse_term))(input)?;
    for num in additions {
        result += num;
    }
    Ok((input, result))
}

pub fn evaluate(input: &str) -> Result<i64, &'static str> {
    let mut buffer = [0u8; 256];
    let new_input = preprocess(input, &mut buffer)?;
    use nom::Finish;
    match parse_expr(new_input).finish() {
        Ok((_remaining, value)) => Ok(value),
        Err(_) => Err("Failed to evaluate expression"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess() {
        let mut buf = [0u8; 128];
        assert_eq!(preprocess(" 1 + 2   *  3 ", &mut buf), Ok("1+2*3"));
        assert_eq!(preprocess("0x10 + 0xA", &mut buf), Ok("16+10"));
        assert_eq!(preprocess("0b101 << 0b11 + 0", &mut buf), Ok("5<<3+0"));
        assert_eq!(preprocess("0b101 << 0xA", &mut buf), Ok("5<<10"));
        let mut small_buf = [0u8; 3];
        assert_eq!(preprocess("0b101 << 0xA", &mut small_buf), Err("Output buffer too small"));
    }

    #[test]
    fn test_parse_factor() {
        assert_eq!(parse_factor("123"), Ok(("", 123)));
        assert_eq!(parse_factor(" 777 "), Ok(("", 777)));
        assert_eq!(parse_factor(" 777 blah blah "), Ok(("blah blah ", 777)));
    }

    #[test]
    fn test_parse_term() {
        assert_eq!(parse_term("1 * 2"), Ok(("", 2)));
        assert_eq!(parse_term("3 * 4 * 5"), Ok(("", 60)));
        assert_eq!(parse_term(" 10 "), Ok(("", 10)));
        assert_eq!(parse_term("7 * 2 + 3"), Ok(("+ 3", 14)));
    }

    #[test]
    fn test_parse_expr() {
        assert_eq!(parse_expr("1 + 2"), Ok(("", 3)));
        assert_eq!(parse_expr("3 + 4 + 5"), Ok(("", 12)));
        assert_eq!(parse_expr("10"), Ok(("", 10)));
        assert_eq!(parse_expr("2 * 3 + 4"), Ok(("", 10)));
        assert_eq!(parse_expr("2 + 3 * 4"), Ok(("", 14)));
        assert_eq!(parse_expr(" 1 + 2 * 3 + 4 "), Ok(("", 11)));
    }

    #[test]
    fn test_evaluate() {
        assert_eq!(evaluate(" 2 * 3 + 4 "), Ok(10));
        assert_eq!(evaluate(" 2 + 3 * 4 "), Ok(14));
        assert_eq!(evaluate(" 1 + 0x10 * 0b10 "), Ok(33));
    }
}
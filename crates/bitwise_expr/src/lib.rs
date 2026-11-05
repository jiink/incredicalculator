#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
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

/// One step of a compiled expression, run on a value stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Const(i64),
    /// The variable `t`.
    T,
    Neg,
    Not,
    Mul,
    Div,
    Rem,
    Add,
    Sub,
    Shl,
    Shr,
    And,
    Xor,
    Or,
}

type Ops = Vec<Op>;

#[derive(Clone, Copy)]
struct Ctx {
    fmt: IntFormat,
    /// Whether the variable `t` may appear.
    allow_t: bool,
}

/// Appends each right-hand side and its operator after `ops`, for a
/// left-associative chain like `a - b - c`.
fn chain(mut ops: Ops, rest: Vec<(Op, Ops)>) -> Ops {
    for (op, rhs) in rest {
        ops.extend(rhs);
        ops.push(op);
    }
    ops
}

fn symbol<'a>(text: &'static str, op: Op) -> impl FnMut(&'a str) -> IResult<&'a str, Op> {
    map(tag(text), move |_| op)
}

fn parse_number(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    map_res(digit1, |s: &str| {
        s.parse::<u64>()
            .map(|v| vec![Op::Const(ctx.fmt.wrap(v as i64))])
    })(input)
}

fn parse_variable(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    if !ctx.allow_t {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    map(tag("t"), |_| vec![Op::T])(input)
}

fn parse_factor(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    alt((
        |i| parse_number(i, ctx),
        |i| parse_variable(i, ctx),
        delimited(tag("("), |i| parse_equation(i, ctx), tag(")")),
        map(preceded(tag("~"), |i| parse_factor(i, ctx)), |mut ops| {
            ops.push(Op::Not);
            ops
        }),
        map(preceded(tag("-"), |i| parse_factor(i, ctx)), |mut ops| {
            ops.push(Op::Neg);
            ops
        }),
    ))(input)
}

fn parse_multiplicative(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    let (input, first) = parse_factor(input, ctx)?;
    let (input, rest) = many0(pair(
        alt((
            symbol("*", Op::Mul),
            symbol("/", Op::Div),
            symbol("%", Op::Rem),
        )),
        |i| parse_factor(i, ctx),
    ))(input)?;
    Ok((input, chain(first, rest)))
}

// alias for the current top level parse
fn parse_equation(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    parse_bitwise_or(input, ctx)
}

fn parse_bitwise_or(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    let (input, first) = parse_bitwise_xor(input, ctx)?;
    let (input, rest) = many0(pair(symbol("|", Op::Or), |i| parse_bitwise_xor(i, ctx)))(input)?;
    Ok((input, chain(first, rest)))
}

fn parse_bitwise_xor(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    let (input, first) = parse_bitwise_and(input, ctx)?;
    let (input, rest) = many0(pair(symbol("^", Op::Xor), |i| parse_bitwise_and(i, ctx)))(input)?;
    Ok((input, chain(first, rest)))
}

fn parse_bitwise_and(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    let (input, first) = parse_shift(input, ctx)?;
    let (input, rest) = many0(pair(symbol("&", Op::And), |i| parse_shift(i, ctx)))(input)?;
    Ok((input, chain(first, rest)))
}

fn parse_shift(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    let (input, first) = parse_additive(input, ctx)?;
    let (input, rest) = many0(pair(
        alt((symbol("<<", Op::Shl), symbol(">>", Op::Shr))),
        |i| parse_additive(i, ctx),
    ))(input)?;
    Ok((input, chain(first, rest)))
}

fn parse_additive(input: &str, ctx: Ctx) -> IResult<&str, Ops> {
    let (input, first) = parse_multiplicative(input, ctx)?;
    let (input, rest) = many0(pair(
        alt((symbol("+", Op::Add), symbol("-", Op::Sub))),
        |i| parse_multiplicative(i, ctx),
    ))(input)?;
    Ok((input, chain(first, rest)))
}

/// Deepest value stack a compiled expression may need.
const MAX_STACK: usize = 64;

/// An expression compiled once so it can be evaluated cheaply many times,
/// such as once per audio sample.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    ops: Ops,
    fmt: IntFormat,
}

impl Program {
    /// Compiles `input`, which may use the variable `t`. Unlike `evaluate`,
    /// the whole input must be a valid expression.
    pub fn compile(input: &str, fmt: IntFormat) -> Result<Program, &'static str> {
        compile_with(input, Ctx { fmt, allow_t: true }, true)
    }

    /// Evaluates the expression with `t` set to `t`, wrapped to the format.
    pub fn eval(&self, t: i64) -> Result<i64, &'static str> {
        let fmt = self.fmt;
        let mut stack = [0i64; MAX_STACK];
        let mut len = 0;
        for &op in &self.ops {
            match op {
                Op::Const(value) => {
                    stack[len] = value;
                    len += 1;
                }
                Op::T => {
                    stack[len] = fmt.wrap(t);
                    len += 1;
                }
                Op::Neg => stack[len - 1] = fmt.wrap(stack[len - 1].wrapping_neg()),
                Op::Not => stack[len - 1] = fmt.wrap(!stack[len - 1]),
                _ => {
                    len -= 1;
                    let (a, b) = (stack[len - 1], stack[len]);
                    let result = match op {
                        Op::Mul => a.wrapping_mul(b),
                        Op::Div if b == 0 => return Err("Division by zero"),
                        Op::Div => a.wrapping_div(b),
                        Op::Rem if b == 0 => return Err("Division by zero"),
                        Op::Rem => a.wrapping_rem(b),
                        Op::Add => a.wrapping_add(b),
                        Op::Sub => a.wrapping_sub(b),
                        Op::Shl => a.wrapping_shl(b as u32),
                        Op::Shr => a.wrapping_shr(b as u32),
                        Op::And => a & b,
                        Op::Xor => a ^ b,
                        Op::Or => a | b,
                        Op::Const(_) | Op::T | Op::Neg | Op::Not => unreachable!(),
                    };
                    stack[len - 1] = fmt.wrap(result);
                }
            }
        }
        Ok(stack[0])
    }
}

fn stack_depth(ops: &[Op]) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0;
    for op in ops {
        match op {
            Op::Const(_) | Op::T => depth += 1,
            Op::Neg | Op::Not => {}
            _ => depth -= 1,
        }
        max_depth = max_depth.max(depth);
    }
    max_depth
}

/// With `require_all` false, input after the first complete expression is
/// ignored, as the calculator has always done.
fn compile_with(input: &str, ctx: Ctx, require_all: bool) -> Result<Program, &'static str> {
    let mut buffer = [0u8; 256];
    let new_input = preprocess(input, &mut buffer)?;
    use nom::Finish;
    match parse_equation(new_input, ctx).finish() {
        Ok((remaining, ops)) => {
            if require_all && !remaining.is_empty() {
                return Err("Failed to evaluate expression");
            }
            if stack_depth(&ops) > MAX_STACK {
                return Err("Expression too complex");
            }
            Ok(Program { ops, fmt: ctx.fmt })
        }
        Err(_) => Err("Failed to evaluate expression"),
    }
}

pub fn evaluate(input: &str) -> Result<i64, &'static str> {
    evaluate_as(input, IntFormat::I64)
}

pub fn evaluate_as(input: &str, fmt: IntFormat) -> Result<i64, &'static str> {
    let ctx = Ctx {
        fmt,
        allow_t: false,
    };
    compile_with(input, ctx, false)?
        .eval(0)
        .map_err(|_| "Failed to evaluate expression")
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

    const I64: Ctx = Ctx {
        fmt: IntFormat::I64,
        allow_t: false,
    };

    /// Runs one of the parsers, returning what it left unparsed and the
    /// value of what it parsed.
    fn run(parser: fn(&str, Ctx) -> IResult<&str, Ops>, input: &str) -> (&str, i64) {
        let (rest, ops) = parser(input, I64).unwrap();
        let program = Program {
            ops,
            fmt: IntFormat::I64,
        };
        (rest, program.eval(0).unwrap())
    }

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
        assert_eq!(run(parse_factor, "123"), ("", 123));
        assert_eq!(run(parse_factor, "777blahblah"), ("blahblah", 777));
    }

    #[test]
    fn test_parse_multiplicative() {
        assert_eq!(run(parse_multiplicative, "1*2"), ("", 2));
        assert_eq!(run(parse_multiplicative, "3*4*5"), ("", 60));
        assert_eq!(run(parse_multiplicative, "10"), ("", 10));
        assert_eq!(run(parse_multiplicative, "7*2+3"), ("+3", 14));
        assert_eq!(run(parse_multiplicative, "10%3"), ("", 1));
        assert_eq!(run(parse_multiplicative, "0%300"), ("", 0));
        assert_eq!(run(parse_multiplicative, "300%300"), ("", 0));
    }

    #[test]
    fn test_parse_additive() {
        assert_eq!(run(parse_additive, "1+2"), ("", 3));
        assert_eq!(run(parse_additive, "3+4+5"), ("", 12));
        assert_eq!(run(parse_additive, "10"), ("", 10));
        assert_eq!(run(parse_additive, "2*3+4"), ("", 10));
        assert_eq!(run(parse_additive, "2+3*4"), ("", 14));
        assert_eq!(run(parse_additive, "1+2*3+4"), ("", 11));
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

    #[test]
    fn programs_evaluate_with_t() {
        let i32_ = IntFormat::new(32, true);
        let program = Program::compile("t*(42&t>>10)", i32_).unwrap();
        assert_eq!(program.eval(3000), Ok(6000));
        assert_eq!(program.eval(5000), Ok(0));

        let program = Program::compile("10*(t>>7|t|t>>6)+4*(t&t>>13|t>>6)", i32_).unwrap();
        for t in [0, 1, 1234, 99_999] {
            let expected = 10 * (t >> 7 | t | t >> 6) + 4 * (t & t >> 13 | t >> 6);
            assert_eq!(program.eval(t), Ok(expected));
        }

        // t wraps like every other value.
        let u8_ = IntFormat::new(8, false);
        assert_eq!(Program::compile("t", u8_).unwrap().eval(300), Ok(44));
        assert_eq!(Program::compile("-t", i32_).unwrap().eval(5), Ok(-5));
        assert_eq!(Program::compile("~t", i32_).unwrap().eval(0), Ok(-1));
    }

    #[test]
    fn programs_reject_bad_input() {
        let i32_ = IntFormat::new(32, true);
        assert!(Program::compile("", i32_).is_err());
        assert!(Program::compile("t+", i32_).is_err());
        assert!(Program::compile("t)", i32_).is_err());
        assert!(Program::compile("tt", i32_).is_err());
        assert_eq!(
            Program::compile("t/(t-4)", i32_).unwrap().eval(4),
            Err("Division by zero")
        );
        assert_eq!(
            Program::compile("t%0", i32_).unwrap().eval(1),
            Err("Division by zero")
        );
    }

    #[test]
    fn evaluate_does_not_accept_t() {
        assert!(evaluate("t").is_err());
        assert_eq!(evaluate("1/0"), Err("Failed to evaluate expression"));
        // Trailing input after a complete expression is still ignored.
        assert_eq!(evaluate("1+2)"), Ok(3));
    }
}

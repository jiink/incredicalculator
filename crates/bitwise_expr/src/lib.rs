#![no_std]

use nom::{bytes::complete::tag, character::complete::{digit1, space0}, combinator::map_res, multi::many0, sequence::{delimited, preceded}, IResult};

fn parse_i64(input: &str) -> IResult<&str, i64> {
    delimited(
        space0,
        map_res(digit1, |s: &str| s.parse::<i64>()),
        space0
    )(input)
}

fn parse_term(input: &str) -> IResult<&str, i64> {
    let (input, mut result) = parse_i64(input)?;
    let (input, multiplications) = many0(preceded(tag("*"), parse_i64))(input)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_i64() {
        assert_eq!(parse_i64("123"), Ok(("", 123)));
        assert_eq!(parse_i64(" 777 "), Ok(("", 777)));
        assert_eq!(parse_i64(" 777 blah blah "), Ok(("blah blah ", 777)));
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
}
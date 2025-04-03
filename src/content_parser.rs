use std::os::raw::c_ushort;

use chrono::NaiveDate;
use winnow::Parser;
use winnow::Result;
use winnow::ascii::multispace0;
use winnow::combinator::delimited;
use winnow::combinator::opt;
use winnow::combinator::preceded;
use winnow::combinator::repeat;
use winnow::combinator::seq;
use winnow::combinator::terminated;
use winnow::error::ParserError;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::kondo::Task;

pub fn parse_task<'s>(input: &mut &'s str) -> Result<Task> {
    let deadline = preceded(multispace0, parse_date_tag).parse_next(input);
    let content = input.trim();

    Ok(Task::new(None, deadline.unwrap(), content))
}

fn parse_date_tag<'s>(input: &mut &'s str) -> Result<NaiveDate> {
    Ok((parse_prefix, parse_date, parse_suffix)
        .parse_next(input)
        .unwrap()
        .1)
}

fn parse_prefix<'s>(input: &mut &'s str) -> Result<char> {
    let c = input
        .next_token()
        .ok_or_else(|| ParserError::from_input(input))?;
    if c != '[' {
        return Err(ParserError::from_input(input));
    }
    Ok(c)
}

fn parse_suffix<'s>(input: &mut &'s str) -> Result<char> {
    let c = input
        .next_token()
        .ok_or_else(|| ParserError::from_input(input))?;
    if c != ']' {
        return Err(ParserError::from_input(input));
    }
    Ok(c)
}

fn parse_date<'s>(input: &mut &'s str) -> Result<NaiveDate> {
    let date = (
        parse_year,
        parse_separator,
        parse_month,
        parse_separator,
        parse_day,
    )
        .parse_next(input)?;
    match NaiveDate::from_ymd_opt(date.0, date.2, date.4) {
        Some(d) => Ok(d),
        None => Err(ParserError::from_input(input)),
    }
}

fn parse_year<'s>(input: &mut &'s str) -> Result<i32> {
    let year: String = repeat(1..=4, parse_digits).parse_next(input)?;
    if year.len() != 4 {
        return Err(ParserError::from_input(input));
    }
    Ok(year.parse::<i32>().unwrap())
}

fn parse_month<'s>(input: &mut &'s str) -> Result<u32> {
    let month: String = repeat(1..=2, parse_digits).parse_next(input)?;
    if month.len() != 2 {
        return Err(ParserError::from_input(input));
    }
    Ok(month.parse::<u32>().unwrap())
}

fn parse_day<'s>(input: &mut &'s str) -> Result<u32> {
    let day: String = repeat(1..=2, parse_digits).parse_next(input)?;
    if day.len() != 2 {
        return Err(ParserError::from_input(input));
    }
    Ok(day.parse::<u32>().unwrap())
}

fn parse_digits<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1.., (('0'..='9'),)).parse_next(input)
}

fn parse_separator<'s>(input: &mut &'s str) -> Result<char> {
    let c = input
        .next_token()
        .ok_or_else(|| ParserError::from_input(input))?;
    match c {
        '/' => Ok(c),
        '-' => Ok(c),
        _ => Err(ParserError::from_input(input)),
    }
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use super::*;

    #[test]
    fn simple_num_test() {
        let mut input = "2025asjkhk";
        let res = parse_digits(&mut input);
        assert_eq!(res.unwrap(), "2025")
    }

    #[test]
    fn simple_year_test() {
        let mut input = "2034";
        let res = parse_year(&mut input);
        assert_eq!(res.unwrap(), 2034)
    }

    #[test]
    fn simple_date_test() {
        let mut input = "2025-03-01";
        let res = parse_date(&mut input);
        assert_eq!(res.unwrap(), NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
    }

    #[test]
    fn simple_parse_test() {
        let mut input = r#"

            [2025-03-31]
            This is a test task.
            "#;

        let task = Task::new(
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
            "This is a test task.",
        );

        let res = parse_task.parse_next(&mut input);
        assert_eq!(res.unwrap(), task)
    }
}

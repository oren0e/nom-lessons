// AOC 2021 day 5 example
use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending},
    combinator::{map, map_res},
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};
use std::str::FromStr;

// Parse a `u32` from the start of the input string
pub fn parse_numbers(input: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(input)
}

// a point in 2D space
#[derive(Debug, Eq, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    fn parse(input: &str) -> IResult<&str, Self> {
        // This parser outputs a (u32, u32).
        // It uses the `parse_numbers` parser
        // and the `separated_pair` combinator
        let parse_two_numbers = separated_pair(parse_numbers, char(','), parse_numbers);

        // Map the (u32, u32) into a Point.
        map(parse_two_numbers, |(x, y)| Point { x, y })(input)
    }
}

// A line spanning two points
#[derive(Debug, Eq, PartialEq)]
pub struct Line(pub Point, pub Point);

impl Line {
    // Parse a line from the input string
    fn parse(input: &str) -> IResult<&str, Self> {
        let parse_arrow = tag(" -> ");

        // Parse two points separated by an arrow
        let parse_points = separated_pair(Point::parse, parse_arrow, Point::parse);

        // If the parse succeeded, put those two points into a Line
        map(parse_points, |(p0, p1)| Line(p0, p1))(input)
    }
}

// Parse the whole aoc day 5 file
pub fn parse_input(s: &str) -> Vec<Line> {
    let (remaining_input, lines) = separated_list1(line_ending, Line::parse)(s).unwrap();
    //assert!(remaining_input.is_empty());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numbers() {
        assert_eq!(Ok(("", 405)), parse_numbers("405"));
        assert_eq!(Ok(("abc", 405)), parse_numbers("405abc"));
    }

    #[test]
    fn test_parse_point() {
        let tests = [
            ("1,2", Point { x: 1, y: 2 }, ""),
            ("1,2asdf", Point { x: 1, y: 2 }, "asdf"),
        ];
        for (input, expected_output, expected_remaining_input) in tests {
            let (remaining_input, output) = Point::parse(input).unwrap();
            assert_eq!(output, expected_output);
            assert_eq!(remaining_input, expected_remaining_input);
        }
    }

    #[test]
    fn test_parse_line() {
        let tests = [
            (
                "0,9 -> 5,9",
                Line(Point { x: 0, y: 9 }, Point { x: 5, y: 9 }),
                "",
            ),
            (
                "0,9 -> 5,9xyz",
                Line(Point { x: 0, y: 9 }, Point { x: 5, y: 9 }),
                "xyz",
            ),
        ];
        for (input, expected_output, expected_remaining_input) in tests {
            let (remaining_input, output) = Line::parse(input).unwrap();
            assert_eq!(remaining_input, expected_remaining_input);
            assert_eq!(output, expected_output);
        }
    }
    #[test]
    fn test_parse_file() {
        let input =
            include_str!("/Users/oepshtain/rust_projects/nom_stuff/text_with_nom/data/input.txt");
        let lines = parse_input(input);
        assert_eq!(lines.len(), 500);
    }
}

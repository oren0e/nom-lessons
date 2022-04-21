use nom::{
    bits::complete::{tag, take},
    combinator::map,
    multi::many0,
    number::complete::be_u16,
    IResult,
};

type BitInput<'a> = (&'a [u8], usize); // a stream of bits grouped in bytes and the next bit to be read

// Take 4 bits from the BitInput.
// Store the output in a u8, because there's no u4 type, and u8
// is the closest-available size.
// 4 bits is called a "nibble" - it is half of a byte
fn take_nibble(i: BitInput) -> IResult<BitInput, u8> {
    // Rust doesn't have a u4 type. How do we store a 4 bit number?
    // Nom's `take` parser solves this by padding your n bits with leading zeroes,
    // and storing them in some uint type like u8, u16 or whichever one you choose.
    //
    // Have to specify some concrete numeric type, otherwise
    // Rust won't know which type of number you're trying to use
    // here. We use usize here, but we could have used any uint type.
    take(4usize)(i)
}

// The bitwise tag parser matches a specific pattern of bits, like "0110", from the input. Again,
// it's a simple idea, but it raises a tricky question: how do we represent a pattern of bits?
// Nom represents the bit pattern using two parameters:
// count: how many bits long the pattern is
// pattern: The pattern itself, padded with leading zeroes to fit into some uint type.
// For example:
// The pattern 101 is represented as (pattern: 00000_101, count: 3).
// The pattern 111000111 is represented as (pattern: 0000000_111000111, count: 9).
// You, the programmer, will choose which uint types to use for pattern and count -- the parser is
// generic over various uint types.
// It's ususally best to just use the smallest uint that fits the value. So, example 1's pattern fits
// in a u8, example 2's fits in a u16, and in both the count fits in a u8.

// This is just a simple wrapper around the `tag` parser, but it makes the
// parameter types concrete instead of generic, so now Rust knows how to actually
// store the pattern
fn parser(pattern: u8, count: u8, input: BitInput) -> IResult<BitInput, u8> {
    tag(pattern, count)(input)
}

// Takes one bit from the input, returning true for 1 and false for 0.
fn take_bit(i: BitInput) -> IResult<BitInput, bool> {
    map(take(1usize), |bits: u8| bits > 0)(i)
}

// Converting byte-streams to bit-streams and back

// Stub example type. Imagine this has to be parsed from individual bits.
struct BitwiseHeader;

// A bit-level parser
fn parse_header(i: BitInput) -> IResult<BitInput, BitwiseHeader> {
    todo!()
}

// Stub example type.
// The header has to be parsed from bits, but the body can be parsed from bytes.
struct Message {
    header: BitwiseHeader,
    body: Vec<u16>,
}

// A byte-level parser that calls a bit-level parser
fn parse_msg(i: &[u8]) -> IResult<&[u8], Message> {
    // The header has to be parsed from bits
    let (i, header) = nom::bits::bits(parse_header)(i)?;
    // But the rest of the message can be parsed from bytes.
    let (i, body) = many0(be_u16)(i)?;
    Ok((i, Message { header, body }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nibble() {
        // Rust number literals let you put underscores wherever you'd like, to
        // enhance readbility. E.g. you can write 1000000 as 1_000_000.
        // We've used it here to visually separate the two u4 values in this u8
        let input = ([0b1010_1111].as_ref(), 0);
        let (_input, actual_nibble) = take_nibble(input).unwrap();
        let expected_nibble = 0b1010;
        assert_eq!(actual_nibble, expected_nibble);
    }

    #[test]
    fn test_tag_wrapper() {
        // The pattern 1111 matches the stream 1111_1111
        assert!(parser(0b1111, 4, (&[0b1111_1111], 0)).is_ok());
        // The pattern 1 matches the stream too
        assert!(parser(0b1, 1, (&[0b1111_1111], 0)).is_ok());
        // The pattern 01 does _not_ match the stream
        assert!(parser(0b1, 2, (&[0b1111_1111], 0)).is_err());
        // The pattern 1111_1110 doesn't match the stream either
        assert!(parser(0b1111_1110, 8, (&[0b1111_1111], 0)).is_err());
    }

    #[test]
    fn test_take_bit() {
        let input = ([0b10101010].as_ref(), 0);
        let (input, first_bit) = take_bit(input).unwrap();
        assert!(first_bit); // First bit is 1
        let (_input, second_bit) = take_bit(input).unwrap();
        assert!(!second_bit); // Second bit is 0
    }
}

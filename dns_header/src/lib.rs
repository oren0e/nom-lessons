/*
                                 1  1  1  1  1  1
  0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|                      ID                       |
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|                    QDCOUNT                    |
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|                    ANCOUNT                    |
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|                    NSCOUNT                    |
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|                    ARCOUNT                    |
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
 */

use std::convert::TryFrom;

use nom::bits::complete::take;
use nom::combinator::map_res;
use nom::IResult;

// All DNS messages start with a Header (both queries and responses!)
// Structure is defined at https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
#[derive(Debug)]
pub struct Header {
    // A 16 bit identifier assigned by the program that generates any kind of
    // query. This identifier is copied in the corresponding reply and can be used
    // by the requester to match up replies to outstanding queries
    pub id: u16,
    // A one bit field that specifies whether this message is a query (0), or a
    // response (1)
    is_query: bool,
    // A four bit field that specifies kind of query in this message. This value
    // is set by the originator of a query and copied into the response.
    opcode: Opcode,
    // This bit is valid in responses, and specifies that the responding name
    // server is an authority for the domain name in question section. Note that
    // the contents of the answer section may have multiple owner names because
    // of aliases. The AA bit corresponds to the name which matches the query name,
    // or the first owner name in the answer section.
    authoritative_answer: bool,
    // Specifies that this message was truncated due to length greater than
    // that permitted on the transmission channel
    truncation: bool,
    // This bit may be set in a query and is copied into the response. if RD
    // is set, it directs the name server to pursue the query recursively.
    // Recursive query support is optional.
    recursion_desired: bool,
    // This bit (sic) is set or cleared in a response, and denotes whether
    // recursive query support is available in the name server.
    recursion_available: bool,
    pub resp_code: ResponseCode,
    // Number of entries in the question section.
    pub question_count: u16,
    // Number of resource records in the answer section.
    pub answer_count: u16,
    // Number of name server resource records in the authority records section.
    pub name_server_count: u16,
    // Number of resource records in the additional records section.
    pub additional_records_count: u16,
}

type BitInput<'a> = (&'a [u8], usize);

// Takes one bit from the BitInput.
// To parse the four flag fields (which are each one bit long),
// we'll use a helper function:
pub fn take_bit(i: BitInput) -> IResult<BitInput, bool> {
    let (i, bit): (BitInput, u8) = take(1u8)(i)?;
    Ok((i, bit != 0))
}

// A four bit field that specifies kind of query in this message
// This value is set by the originator of a query and copied into the response.
#[derive(Debug)]
enum Opcode {
    // 0: a standard query (QUERY)
    Query,
    // 1: an inverse query (IQUERY)
    InverseQuery,
    // 2: a server status request (STATUS)
    Status,
}

impl TryFrom<u8> for Opcode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let op = match value {
            0 => Self::Query,
            1 => Self::InverseQuery,
            2 => Self::Status,
            other => anyhow::bail!("Unknown opcode {other}"),
        };
        Ok(op)
    }
}

// We also need to parse 4-bit numbers from bit-streams:
// A "nibble" is half a byte, i.e. 4-bit number.
pub fn take_nibble(i: BitInput) -> IResult<BitInput, u8> {
    take(4u8)(i)
}

// Then we can easily parse the opcode by parsing the 4-bit number, and tying to
// convert it into the Opcode enum.
// let (i, opcode) = map_res(take_nibble, Opcode::try_from)(i)?; // map_res applies a function
// which return a Result, over the result of a parser.

// Once you know the size of each field, and you have a struct to represent them all, it's actually
// pretty easy to parse the protocol.

// Take 16 bits from the BitInput, parse intoa uint with most significant bit first
pub fn take_u16(i: BitInput) -> IResult<BitInput, u16> {
    take(16u8)(i)
}

impl Header {
    pub fn deserialize(i: BitInput) -> IResult<BitInput, Self> {
        let (i, id) = take_u16(i)?;
        let (i, qr) = take_bit(i)?;
        let (i, opcode) = map_res(take_nibble, Opcode::try_from)(i)?;
        let (i, aa) = take_bit(i)?;
        let (i, tc) = take_bit(i)?;
        let (i, rd) = take_bit(i)?;
        let (mut i, ra) = take_bit(i)?;
        // The spec defines the Z field as three consecutive 0s.
        for _ in 0..3 {
            let z;
            (i, z) = take_bit(i)?;
            assert!(!z);
        }
        let (i, rcode) = map_res(take_nibble, ResponseCode::try_from)(i)?; // ResponseCode unimplemented here
        let (i, qdcount) = take_u16(i)?;
        let (i, ancount) = take_u16(i)?;
        let (i, nscount) = take_u16(i)?;
        let (i, arcount) = take_u16(i)?;
        let header = Header {
            id,
            is_query: qr,
            opcode,
            authoritative_answer: aa,
            truncation: tc,
            recursion_desired: rd,
            recursion_available: ra,
            resp_code: rcode,
            question_count: qdcount,
            answer_count: ancount,
            name_server_count: nscount,
            additional_records_count: arcount,
        };
        Ok((i, header))
    }
}

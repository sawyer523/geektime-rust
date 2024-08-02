use std::num::NonZeroUsize;
use winnow::{dispatch, Parser, PResult};
use winnow::ascii::{dec_int, float};
use winnow::combinator::{alt, fail, preceded, terminated};
use winnow::error::{ContextError, ErrMode, Needed};
use winnow::token::{any, take, take_until};

use crate::{Array, BulkString, Map, Null, RespError, RespFrame, Set, SimpleError, SimpleString};

const CRLF: &[u8] = b"\r\n";

pub fn parse_frame_length(input: &[u8]) -> Result<usize, RespError> {
    let target = &mut (&*input);
    let ret = parse_frame_len(target);
    match ret {
        Ok(_) => {
            // calculate the distance between target and input
            let start = input.as_ptr() as usize;
            let end = (*target).as_ptr() as usize;
            let len = end - start;
            Ok(len)
        }
        Err(_) => Err(RespError::NotComplete),
    }
}

fn parse_frame_len(input: &mut &[u8]) -> PResult<()> {
    let mut simple_parser = terminated(take_until(0.., CRLF), CRLF).value(());
    dispatch! {any;
        b'+' => simple_parser,
        b'-' => simple_parser,
        b':' => simple_parser,
        b'$' => bulk_string_len,
        b'*' => array_len,
        b'_' => simple_parser,
        b'#' => simple_parser,
        b',' => simple_parser,
        b'%' => map_len,
        b'~' => set_len,
        _v => fail::<_, _, _>
    }
    .parse_next(input)
}

pub fn parse_frame(input: &mut &[u8]) -> PResult<RespFrame> {
    dispatch!(any;
        b'+' => simple_string.map(RespFrame::SimpleString),
        b'-' => error.map(RespFrame::Error),
        b':' => integer.map(RespFrame::Integer),
        b'$' => bulk_string.map(RespFrame::BulkString),
        b'*' => array.map(RespFrame::Array),
        b'_' => null.map(RespFrame::Null),
        b'#' => boolean.map(RespFrame::Boolean),
        b',' => double.map(RespFrame::Double),
        b'%' => map.map(RespFrame::Map),
        b'~' => set.map(RespFrame::Set),
        _v => fail::<_, _, _>
    )
    .parse_next(input)
}

// - simple string: "+OK\r\n"
fn simple_string(input: &mut &[u8]) -> PResult<SimpleString> {
    parse_string.map(SimpleString).parse_next(input)
}

// - error: "-ERR unknown command 'foobar'\r\n"
fn error(input: &mut &[u8]) -> PResult<SimpleError> {
    parse_string.map(SimpleError).parse_next(input)
}

fn integer(input: &mut &[u8]) -> PResult<i64> {
    terminated(dec_int, CRLF).parse_next(input)
}

fn bulk_string_len(input: &mut &[u8]) -> PResult<()> {
    let len = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(());
    } else if len < -1 {
        return Err(err_cut("bulk string length must be non-negative"));
    }

    let len_with_crlf = len as usize + 2;
    if input.len() < len_with_crlf {
        let size = NonZeroUsize::new((len_with_crlf - input.len()) as usize).unwrap();
        return Err(ErrMode::Incomplete(Needed::Size(size)));
    }
    *input = &input[(len + 2) as usize..];
    Ok(())
}

fn bulk_string(input: &mut &[u8]) -> PResult<BulkString> {
    let len: i64 = integer.parse_next(input)?;
    if len < 0 {
        return Ok(BulkString::none());
    } else if len == 0 {
        return Ok(BulkString::new(vec![]));
    }

    let len = len as usize;
    let data = terminated(take(len), CRLF).parse_next(input)?;
    Ok(BulkString::new(data))
}

fn array_len(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(());
    } else if len < -1 {
        return Err(err_cut("array length must be non-negative"));
    }
    for _ in 0..len {
        parse_frame_len(input)?;
    }
    Ok(())
}

#[allow(clippy::comparison_chain)]
fn array(input: &mut &[u8]) -> PResult<Array> {
    let len: i64 = integer.parse_next(input)?;
    if len < 0 {
        return Ok(Array::none());
    } else if len == 0 {
        return Ok(Array::new(vec![]));
    }

    let len = len as usize;
    let mut frames = Vec::with_capacity(len);
    for _ in 0..len {
        frames.push(parse_frame(input)?);
    }
    Ok(Array::new(frames))
}

fn null(input: &mut &[u8]) -> PResult<Null> {
    CRLF.value(Null).parse_next(input)
}

fn boolean(input: &mut &[u8]) -> PResult<bool> {
    let b = alt(('t', 'f')).parse_next(input)?;
    Ok(b == 't')
}

// - float: ",3.14\r\n"
fn double(input: &mut &[u8]) -> PResult<f64> {
    terminated(float, CRLF).parse_next(input)
}

// - map: "%2\r\n+foo\r\n+bar\r\n"
fn map(input: &mut &[u8]) -> PResult<Map> {
    let len: i64 = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("map length must be greater than 0"));
    }

    let len = len;
    let mut map = Map::new();
    for _ in 0..len {
        let key = preceded('+', parse_string).parse_next(input)?;
        let value = parse_frame(input)?;
        map.insert(key, value);
    }
    Ok(map)
}

fn map_len(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("map length must be non-negative"));
    }
    for _ in 0..len {
        terminated(take_until(0.., CRLF), CRLF)
            .value(())
            .parse_next(input)?;
        parse_frame_len(input)?;
    }
    Ok(())
}

fn set(input: &mut &[u8]) -> PResult<Set> {
    let len: i64 = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("set length must be greater than 0"));
    }

    let len = len as usize;
    let mut set = Vec::new();
    for _ in 0..len {
        let v = parse_frame(input)?;
        set.push(v);
    }
    Ok(Set::new(set))
}

fn set_len(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("set length must be non-negative"));
    }
    for _ in 0..len {
        parse_frame_len(input)?;
    }
    Ok(())
}

fn parse_string(input: &mut &[u8]) -> PResult<String> {
    terminated(take_until(0.., CRLF), CRLF)
        .map(|s: &[u8]| String::from_utf8_lossy(s).into_owned())
        .parse_next(input)
}

fn err_cut(_s: impl Into<String>) -> ErrMode<ContextError> {
    let context = ContextError::default();
    ErrMode::Cut(context)
}

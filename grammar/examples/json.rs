use anyhow::{anyhow, Result};
use winnow::{Parser, PResult};
use winnow::ascii::{dec_int, float, multispace0};
use winnow::combinator::{alt, delimited, separated, separated_pair, trace};
use winnow::error::{ContextError, ErrMode, ParserError};
use winnow::stream::{AsChar, Stream, StreamIsPartial};
use winnow::token::take_until;

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(std::collections::HashMap<String, JsonValue>),
}

fn main() -> Result<()> {
    let str = r#"{"name": "John Doe", "age": 43, "is_student": false, "marks": [90, -80, 85], "address": {"city": "New York", "zip": 10001}}"#;
    let person = parse_json(str)?;
    println!("{:#?}", person);
    Ok(())
}

fn parse_json(s: &str) -> Result<JsonValue> {
    let input = &mut (&*s);
    parse_value(input).map_err(|e: ErrMode<ContextError>| anyhow!("Failed to parse JSON: {:?}", e))
}

pub fn sep_with_space<Input, Output, Error, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<Input, (), Error>
where
    Input: Stream + StreamIsPartial,
    <Input as Stream>::Token: AsChar + Clone,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
{
    trace("sep_with_space", move |input: &mut Input| {
        let _ = multispace0.parse_next(input)?;
        let o2 = parser.parse_next(input)?;
        multispace0.parse_next(input).map(|_| o2)?;
        Ok(())
    })
}

fn parse_null(input: &mut &str) -> PResult<()> {
    "null".value(()).parse_next(input)
}

fn parse_bool(input: &mut &str) -> PResult<bool> {
    alt(("true", "false")).parse_to().parse_next(input)
}

fn parse_string(input: &mut &str) -> PResult<String> {
    let ret = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(ret.to_string())
}

fn parse_array(input: &mut &str) -> PResult<Vec<JsonValue>> {
    let sep1 = sep_with_space('[');
    let sep2 = sep_with_space(']');
    let sep_comma = sep_with_space(',');
    let parse_one_value = separated(0.., parse_value, sep_comma);
    delimited(sep1, parse_one_value, sep2).parse_next(input)
}

fn parse_object(input: &mut &str) -> PResult<std::collections::HashMap<String, JsonValue>> {
    let sep1 = sep_with_space('{');
    let sep2 = sep_with_space('}');
    let sep_comma = sep_with_space(',');
    let sep_colon = sep_with_space(':');
    let parse_kv_pair = separated_pair(parse_string, sep_colon, parse_value);
    let parse_kv = separated(1.., parse_kv_pair, sep_comma);
    delimited(sep1, parse_kv, sep2).parse_next(input)
}

fn parse_value(input: &mut &str) -> PResult<JsonValue> {
    alt((
        parse_null.value(JsonValue::Null),
        parse_bool.map(JsonValue::Bool),
        alt((dec_int.map(JsonValue::Int), float.map(JsonValue::Number))),
        parse_string.map(JsonValue::String),
        parse_array.map(JsonValue::Array),
        parse_object.map(JsonValue::Object),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::error::ContextError;

    use super::*;

    #[test]
    fn test_parse_null() -> PResult<(), ContextError> {
        let mut input = "null";
        let ret = parse_null(&mut input)?;
        assert_eq!(ret, ());
        assert_eq!(input, "");

        Ok(())
    }

    #[test]
    fn test_parse_bool() -> PResult<(), ContextError> {
        let mut input = "true";
        let ret = parse_bool(&mut input)?;
        assert_eq!(ret, true);
        assert_eq!(input, "");

        let mut input = "false";
        let ret = parse_bool(&mut input)?;
        assert_eq!(ret, false);
        assert_eq!(input, "");

        Ok(())
    }

    #[test]
    fn test_parse_string() -> PResult<(), ContextError> {
        let mut input = r#""hello""#;
        let ret = parse_string(&mut input)?;
        assert_eq!(ret, "hello");

        Ok(())
    }

    #[test]
    fn test_parse_array() -> PResult<(), ContextError> {
        let mut input = r#"[1, 2, 3]"#;
        let ret = parse_array(&mut input)?;
        assert_eq!(
            ret,
            vec![JsonValue::Int(1), JsonValue::Int(2), JsonValue::Int(3)]
        );

        Ok(())
    }

    #[test]
    fn test_parse_object() -> PResult<(), ContextError> {
        let mut input = r#"{"name": "John Doe", "age": 43}"#;
        let ret = parse_object(&mut input)?;
        let mut map = std::collections::HashMap::new();
        map.insert(
            "name".to_string(),
            JsonValue::String("John Doe".to_string()),
        );
        map.insert("age".to_string(), JsonValue::Int(43));
        assert_eq!(ret, map);

        Ok(())
    }
}

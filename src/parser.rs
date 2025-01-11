use crate::ir::{Array, Document, Identifier, InlineTable, Pair, Table, Value};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{alphanumeric1, char, digit1, multispace0, newline, space0},
    combinator::{eof, map_res, not, opt},
    error::Error as NomError,
    multi::{fold_many0, fold_many1, separated_list0},
    number::complete::double,
    sequence::{delimited, pair, separated_pair, tuple},
    Finish, IResult, Parser,
};

fn parse_float(s: &str) -> IResult<&str, f64> {
    double(s)
}

// fn parse_integer(s: &str) -> IResult<&str, i64> {
//     map_res(digit1, str::parse)(s)
// }

fn parse_integer(s: &str) -> IResult<&str, i64> {
    // explain why using and, not and_then
    let integer = map_res(digit1, str::parse);
    let not_float = not(tuple((digit1, char('.'), digit1)));
    not_float.and(integer).map(|(_, i)| i).parse(s)
}

fn parse_boolean(s: &str) -> IResult<&str, bool> {
    alt((tag("true"), tag("false")))
        .map(|s: &str| match s {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        })
        .parse(s)
}

fn parse_string(s: &str) -> IResult<&str, String> {
    delimited(char('"'), take_till(|c| c == '"'), char('"'))
        .map(|s: &str| s.to_string())
        .parse(s)
}

fn parse_array(s: &str) -> IResult<&str, Array> {
    let sep = tuple((multispace0, char(','), multispace0));
    let par = separated_list0(sep, parse_value);
    delimited(
        pair(char('['), multispace0),
        par,
        pair(multispace0, char(']')),
    )
    .map(Array)
    .parse(s)
}

fn parse_identifier(s: &str) -> IResult<&str, Identifier> {
    let par = alt((alphanumeric1, tag("-"), tag("_")));
    fold_many1(par, String::new, |string, s| string + s)
        .map(Identifier)
        .parse(s)
}

fn parse_pair(s: &str) -> IResult<&str, Pair> {
    let sep = tuple((space0, char('='), space0));
    separated_pair(parse_identifier, sep, parse_value)
        .map(|(key, value)| Pair { key, value })
        .parse(s)
}

fn parse_inline_table(s: &str) -> IResult<&str, InlineTable> {
    let sep = tuple((multispace0, char(','), multispace0));
    let par = separated_list0(sep, parse_pair);
    delimited(
        pair(char('{'), multispace0),
        par,
        pair(multispace0, char('}')),
    )
    .map(InlineTable)
    .parse(s)
}

fn parse_value(s: &str) -> IResult<&str, Value> {
    alt((
        parse_boolean.map(Value::Boolean),
        parse_integer.map(Value::Integer),
        parse_float.map(Value::Float),
        parse_string.map(Value::String),
        parse_array.map(Value::Array),
        parse_inline_table.map(Value::InlineTable),
    ))
    .parse(s)
}

fn parse_table_body(s: &str) -> IResult<&str, InlineTable> {
    let par = tuple((multispace0, parse_pair, space0)).map(|(_, p, _)| p);
    separated_list0(newline, par).map(InlineTable).parse(s)
}

fn parse_table(s: &str) -> IResult<&str, Table> {
    let header = tuple((
        multispace0,
        char('['),
        space0,
        parse_identifier,
        space0,
        char(']'),
        space0,
        newline,
    ))
    .map(|(_, _, _, i, _, _, _, _)| i);

    pair(header, parse_table_body)
        .map(|(header, body)| Table { header, body })
        .parse(s)
}

fn parse_document(s: &str) -> IResult<&str, Document> {
    let par = fold_many0(parse_table, Vec::new, |mut vec, t| {
        vec.push(t);
        vec
    });
    tuple((opt(parse_table_body), par, multispace0, eof))
        .map(|(opt, mut vec, _, _)| {
            if let Some(body) = opt {
                let header = Identifier(String::new());
                vec.insert(0, Table { header, body });
            }
            Document(vec)
        })
        .parse(s)
}

pub fn parse(s: &str) -> Result<Document, NomError<&str>> {
    parse_document(s).finish().map(|(_, vec)| vec)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use insta::{assert_compact_debug_snapshot, assert_debug_snapshot};

    #[test]
    fn test_parse_boolean_1() {
        let r = parse_boolean("false").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", false)"#)
    }

    #[test]
    fn test_parse_boolean_2() {
        let r = parse_boolean("true").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", true)"#)
    }

    #[test]
    fn test_parse_boolean_3() {
        let _ = parse_boolean("other").unwrap_err();
    }

    #[test]
    fn test_parse_integer_1() {
        let r = parse_integer("1").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", 1)"#)
    }

    #[test]
    fn test_parse_integer_2() {
        let r = parse_integer("1other").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("other", 1)"#)
    }

    #[test]
    fn test_parse_float_1() {
        let r = parse_float("1.0").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", 1.0)"#)
    }

    #[test]
    fn test_parse_float_2() {
        let r = parse_float("0.1").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", 0.1)"#)
    }

    #[test]
    fn test_parse_float_3() {
        let r = parse_float("0.1other").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("other", 0.1)"#)
    }

    #[test]
    fn test_parse_string_1() {
        let r = parse_string("\"abc\"").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", "abc")"#)
    }

    #[test]
    fn test_parse_string_2() {
        let r = parse_string("\"abc\"other").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("other", "abc")"#)
    }

    #[test]
    fn test_parse_array_1() {
        let r = parse_array("[1,2]").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", Array([Integer(1), Integer(2)]))"#)
    }

    #[test]
    fn test_parse_array_2() {
        let r = parse_array("[  \"abc\", 1, 2.0, true ]").unwrap();
        assert_debug_snapshot!(r, @r#"
        (
            "",
            Array(
                [
                    String(
                        "abc",
                    ),
                    Integer(
                        1,
                    ),
                    Float(
                        2.0,
                    ),
                    Boolean(
                        true,
                    ),
                ],
            ),
        )
        "#)
    }

    #[test]
    fn test_parse_identifier_1() {
        let r = parse_identifier("abc").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", Identifier("abc"))"#)
    }

    #[test]
    fn test_parse_identifier_2() {
        let r = parse_identifier("-ab_c").unwrap();
        assert_compact_debug_snapshot!(r, @r#"("", Identifier("-ab_c"))"#)
    }

    #[test]
    fn test_parse_pair_1() {
        let r = parse_pair("abc=\"def\"").unwrap();
        assert_debug_snapshot!(r, @r#"
        (
            "",
            Pair {
                key: Identifier(
                    "abc",
                ),
                value: String(
                    "def",
                ),
            },
        )
        "#)
    }

    #[test]
    fn test_parse_pair_2() {
        let r = parse_pair("abc = \"def\"").unwrap();
        assert_debug_snapshot!(r, @r#"
        (
            "",
            Pair {
                key: Identifier(
                    "abc",
                ),
                value: String(
                    "def",
                ),
            },
        )
        "#)
    }

    #[test]
    fn test_parse_inline_table_1() {
        let r = parse_inline_table("{ abc = \"def\" }").unwrap();
        assert_debug_snapshot!(r, @r#"
        (
            "",
            InlineTable(
                [
                    Pair {
                        key: Identifier(
                            "abc",
                        ),
                        value: String(
                            "def",
                        ),
                    },
                ],
            ),
        )
        "#)
    }

    pub const TOML: &str = r#"
title = "TOML Example"

[owner]
name = "Tom Preston-Werner"

[database]
enabled = true
ports = [ 8000, 8001, 8002 ]
data = [ ["delta", "phi"], [3.14] ]
temp_targets = { cpu = 79.5, case = 72.0 }

[servers-alpha]
ip = "10.0.0.1"
role = "frontend"

[servers-beta]
ip = "10.0.0.2"
role = "backend""#;

    #[test]
    fn test_parse_document() {
        let r = parse_document(TOML).unwrap();
        assert_debug_snapshot!(r, @r#"
        (
            "",
            Document(
                [
                    Table {
                        header: Identifier(
                            "",
                        ),
                        body: InlineTable(
                            [
                                Pair {
                                    key: Identifier(
                                        "title",
                                    ),
                                    value: String(
                                        "TOML Example",
                                    ),
                                },
                            ],
                        ),
                    },
                    Table {
                        header: Identifier(
                            "owner",
                        ),
                        body: InlineTable(
                            [
                                Pair {
                                    key: Identifier(
                                        "name",
                                    ),
                                    value: String(
                                        "Tom Preston-Werner",
                                    ),
                                },
                            ],
                        ),
                    },
                    Table {
                        header: Identifier(
                            "database",
                        ),
                        body: InlineTable(
                            [
                                Pair {
                                    key: Identifier(
                                        "enabled",
                                    ),
                                    value: Boolean(
                                        true,
                                    ),
                                },
                                Pair {
                                    key: Identifier(
                                        "ports",
                                    ),
                                    value: Array(
                                        Array(
                                            [
                                                Integer(
                                                    8000,
                                                ),
                                                Integer(
                                                    8001,
                                                ),
                                                Integer(
                                                    8002,
                                                ),
                                            ],
                                        ),
                                    ),
                                },
                                Pair {
                                    key: Identifier(
                                        "data",
                                    ),
                                    value: Array(
                                        Array(
                                            [
                                                Array(
                                                    Array(
                                                        [
                                                            String(
                                                                "delta",
                                                            ),
                                                            String(
                                                                "phi",
                                                            ),
                                                        ],
                                                    ),
                                                ),
                                                Array(
                                                    Array(
                                                        [
                                                            Float(
                                                                3.14,
                                                            ),
                                                        ],
                                                    ),
                                                ),
                                            ],
                                        ),
                                    ),
                                },
                                Pair {
                                    key: Identifier(
                                        "temp_targets",
                                    ),
                                    value: InlineTable(
                                        InlineTable(
                                            [
                                                Pair {
                                                    key: Identifier(
                                                        "cpu",
                                                    ),
                                                    value: Float(
                                                        79.5,
                                                    ),
                                                },
                                                Pair {
                                                    key: Identifier(
                                                        "case",
                                                    ),
                                                    value: Float(
                                                        72.0,
                                                    ),
                                                },
                                            ],
                                        ),
                                    ),
                                },
                            ],
                        ),
                    },
                    Table {
                        header: Identifier(
                            "servers-alpha",
                        ),
                        body: InlineTable(
                            [
                                Pair {
                                    key: Identifier(
                                        "ip",
                                    ),
                                    value: String(
                                        "10.0.0.1",
                                    ),
                                },
                                Pair {
                                    key: Identifier(
                                        "role",
                                    ),
                                    value: String(
                                        "frontend",
                                    ),
                                },
                            ],
                        ),
                    },
                    Table {
                        header: Identifier(
                            "servers-beta",
                        ),
                        body: InlineTable(
                            [
                                Pair {
                                    key: Identifier(
                                        "ip",
                                    ),
                                    value: String(
                                        "10.0.0.2",
                                    ),
                                },
                                Pair {
                                    key: Identifier(
                                        "role",
                                    ),
                                    value: String(
                                        "backend",
                                    ),
                                },
                            ],
                        ),
                    },
                ],
            ),
        )
        "#);
    }
}

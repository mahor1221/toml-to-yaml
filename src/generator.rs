use crate::ir::{Array, Document, Identifier, InlineTable, Pair, Table, Value};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

const INDENTATION: &str = "  ";

// It's better to use a custom trait named like DisplayYaml instead of Display

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Integer(v) => v.fmt(f),
            // See: https://doc.rust-lang.org/std/fmt/index.html
            Self::Float(v) => write!(f, "{:?}", v),
            Self::Boolean(v) => v.fmt(f),
            Self::String(v) => v.fmt(f),
            Self::Array(v) => indent_inbetween(f, &v.to_string()),
            Self::InlineTable(v) => indent_inbetween(f, &v.to_string()),
        }
    }
}

impl Display for Array {
    // puts hyphen before each array item and
    // puts newline between array items
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut iter = self.0.iter();
        if let Some(value) = iter.next() {
            f.write_str("- ")?;
            value.fmt(f)?;
        }
        for value in iter {
            f.write_char('\n')?;
            f.write_str("- ")?;
            value.fmt(f)?;
        }

        Ok(())
    }
}

impl Display for InlineTable {
    // puts newline between table pairs
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut iter = self.0.iter();
        if let Some(pair) = iter.next() {
            pair.fmt(f)?;
        }
        for pair in iter {
            f.write_char('\n')?;
            pair.fmt(f)?;
        }

        Ok(())
    }
}

impl Display for Pair {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Self { key, value } = self;

        key.fmt(f)?;
        f.write_char(':')?;

        use Value::*;
        match value {
            Integer(_) | Float(_) | Boolean(_) | String(_) => {
                f.write_char(' ')?;
            }
            InlineTable(_) | Array(_) => {
                f.write_char('\n')?;
                f.write_str(INDENTATION)?;
            }
        }

        value.fmt(f)
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Self { header, body } = self;

        if header.0.is_empty() {
            body.fmt(f)
        } else {
            header.fmt(f)?;
            f.write_str(":\n")?;
            indent_all(f, &body.to_string())
        }
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut iter = self.0.iter();
        if let Some(table) = iter.next() {
            table.fmt(f)?;
        }
        for table in iter {
            f.write_str("\n\n")?;
            table.fmt(f)?;
        }

        Ok(())
    }
}

// puts indentation between lines
fn indent_inbetween(f: &mut Formatter<'_>, s: &str) -> FmtResult {
    let mut iter = s.split_inclusive("\n");
    if let Some(line) = iter.next() {
        f.write_str(line)?;
    }
    for line in iter {
        f.write_str(INDENTATION)?;
        f.write_str(line)?;
    }

    Ok(())
}

// puts indentation before each line
fn indent_all(f: &mut Formatter<'_>, s: &str) -> FmtResult {
    for line in s.split_inclusive("\n") {
        f.write_str(INDENTATION)?;
        f.write_str(line)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::parser::{parse, test::TOML};
    use insta::assert_snapshot;

    #[test]
    fn test_display_yaml() {
        let doc = parse(TOML).unwrap();
        let r = doc.to_string();

        assert_snapshot!(r, @r"
        title: TOML Example

        owner:
          name: Tom Preston-Werner

        database:
          enabled: true
          ports:
            - 8000
            - 8001
            - 8002
          data:
            - - delta
              - phi
            - - 3.14
              - a: 72.0
                b: 26
          temp_targets:
            cpu: 79.5
            case:
              a: 72.0
              b: 26

        servers-alpha:
          ip: 10.0.0.1
          role: frontend

        servers-beta:
          ip: 10.0.0.2
          role: backend
        ")
    }
}

use crate::ir::{Array, Document, Identifier, InlineTable, Pair, Table, Value};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Integer(v) => v.fmt(f),
            Self::Float(v) => v.fmt(f),
            Self::Boolean(v) => v.fmt(f),
            Self::String(v) => v.fmt(f),
            Self::Array(v) => v.fmt(f),
            Self::InlineTable(v) => v.fmt(f),
        }
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_char('[')?;

        if let Some(value) = self.0.first() {
            value.fmt(f)?;
        }
        for value in self.0.iter().skip(1) {
            f.write_str(", ")?;
            value.fmt(f)?;
        }

        f.write_char(']')
    }
}

impl Display for InlineTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(pair) = self.0.first() {
            pair.fmt(f)?;
        }
        for pair in self.0.iter().skip(1) {
            f.write_char('\n')?;
            pair.fmt(f)?;
        }

        Ok(())
    }
}

impl Display for Pair {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Self { key, value } = self;

        match value {
            Value::Integer(_)
            | Value::Float(_)
            | Value::Boolean(_)
            | Value::String(_)
            | Value::Array(_) => {
                key.fmt(f)?;
                f.write_str(": ")?;
                value.fmt(f)
            }
            Value::InlineTable(_) => {
                key.fmt(f)?;
                f.write_str(":\n")?;
                write_indented(f, &value.to_string())
            }
        }
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
            write_indented(f, &body.to_string())
        }
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(table) = self.0.first() {
            table.fmt(f)?;
        }
        for table in self.0.iter().skip(1) {
            f.write_char('\n')?;
            table.fmt(f)?;
        }

        Ok(())
    }
}

const INDENTATION: &str = "  ";
fn write_indented(f: &mut Formatter<'_>, s: &str) -> FmtResult {
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
          ports: [8000, 8001, 8002]
          data: [[delta, phi], [3.14]]
          temp_targets:
            cpu: 79.5
            case: 72
        servers-alpha:
          ip: 10.0.0.1
          role: frontend
        servers-beta:
          ip: 10.0.0.2
          role: backend
        ")
    }
}

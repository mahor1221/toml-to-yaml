#[derive(Debug, PartialEq)]
pub struct Array(pub Vec<Value>);

#[derive(Debug, PartialEq)]
pub struct InlineTable(pub Vec<Pair>);

#[derive(Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Array),
    InlineTable(InlineTable),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Identifier(pub String);

// pub struct Identifier {
//     pub inner: String
// }

// impl Identifier {
//     pub fn new(s: String) -> Self {
//         Self(s)
//     }
// }

// fn main() {
//     let string = String::from("abcd");

//     // public
//     let ident = Identifier(string);
//     let a = ident.0;
//     // or
//     let a = ident.inner;

//     // private
//     let ident = Identifier::new(string);
//     let a = ident.as_string(); // better to implement AsRef or From instead
// }

#[derive(Debug, PartialEq)]
pub struct Pair {
    pub key: Identifier,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub struct Table {
    pub header: Identifier,
    pub body: InlineTable,
}

#[derive(Debug, PartialEq)]
pub struct Document(pub Vec<Table>);

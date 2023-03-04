/// InfluxDB Line Protocol escaping helper module.
/// https://docs.influxdata.com/influxdb/v1.7/write_protocols/line_protocol_tutorial/
use crate::Type;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref COMMAS_SPACES: Regex = Regex::new("[, ]").unwrap();
    pub static ref COMMAS_SPACES_EQUALS: Regex = Regex::new("[, =]").unwrap();
    pub static ref QUOTES_SLASHES: Regex = Regex::new(r#"["\\]"#).unwrap();
    pub static ref SLASHES: Regex = Regex::new(r#"(\\|,| |=|")"#).unwrap();
}

pub enum LineProtoTerm<'a> {
    Measurement(&'a str), // escape commas, spaces
    TagKey(&'a str),      // escape commas, equals, spaces
    TagValue(&'a Type),   // escape commas, equals, spaces
    FieldKey(&'a str),    // escape commas, equals, spaces
    FieldValue(&'a Type), // escape quotes, backslashes + quote
}

impl LineProtoTerm<'_> {
    pub fn escape(self) -> String {
        use LineProtoTerm::*;
        match self {
            Measurement(x) => Self::escape_any(x, &COMMAS_SPACES),
            TagKey(x) | FieldKey(x) => Self::escape_any(x, &COMMAS_SPACES_EQUALS),
            FieldValue(x) => Self::escape_field_value(x),
            TagValue(x) => Self::escape_tag_value(x),
        }
    }

    fn escape_field_value(v: &Type) -> String {
        use Type::*;
        match v {
            Boolean(v) => {
                if *v {
                    "true"
                } else {
                    "false"
                }
            }
            .to_string(),
            Float(v) => v.to_string(),
            SignedInteger(v) => format!("{}i", v),
            UnsignedInteger(v) => format!("{}u", v),
            Text(v) => format!(r#""{}""#, Self::escape_any(v, &QUOTES_SLASHES)),
        }
    }

    fn escape_tag_value(v: &Type) -> String {
        use Type::*;
        match v {
            Boolean(v) => {
                if *v {
                    "true"
                } else {
                    "false"
                }
            }
            .to_string(),
            Float(v) => format!(r#"{}"#, v),
            SignedInteger(v) => format!(r#"{}"#, v),
            UnsignedInteger(v) => format!(r#"{}"#, v),
            Text(v) => Self::escape_any(v, &SLASHES),
        }
    }

    fn escape_any(s: &str, re: &Regex) -> String {
        re.replace_all(s, r#"\$0"#).to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::query::line_proto_term::LineProtoTerm::*;
    use crate::Type;

    #[test]
    fn test() {
        assert_eq!(TagValue(&Type::Boolean(true)).escape(), r#"true"#);
        assert_eq!(TagValue(&Type::Float(1.8324f64)).escape(), r#"1.8324"#);
        assert_eq!(TagValue(&Type::SignedInteger(-1i64)).escape(), r#"-1"#);
        assert_eq!(TagValue(&Type::UnsignedInteger(1u64)).escape(), r#"1"#);

        assert_eq!(
            TagValue(&Type::Text("this is my special string".into())).escape(),
            r#"this\ is\ my\ special\ string"#
        );
        assert_eq!(
            TagValue(&Type::Text("a tag w=i th == tons of escapes".into())).escape(),
            r#"a\ tag\ w\=i\ th\ \=\=\ tons\ of\ escapes"#
        );
        assert_eq!(
            TagValue(&Type::Text("no_escapes".into())).escape(),
            r#"no_escapes"#
        );
        assert_eq!(
            TagValue(&Type::Text("some,commas,here".into())).escape(),
            r#"some\,commas\,here"#
        );

        assert_eq!(Measurement(r#"wea", ther"#).escape(), r#"wea"\,\ ther"#);
        assert_eq!(TagKey(r#"locat\ ,=ion"#).escape(), r#"locat\\ \,\=ion"#);

        assert_eq!(FieldValue(&Type::Boolean(true)).escape(), r#"true"#);
        assert_eq!(FieldValue(&Type::Boolean(false)).escape(), r#"false"#);

        assert_eq!(FieldValue(&Type::Float(0.0)).escape(), r#"0"#);
        assert_eq!(FieldValue(&Type::Float(-0.1)).escape(), r#"-0.1"#);

        assert_eq!(FieldValue(&Type::SignedInteger(0)).escape(), r#"0i"#);
        assert_eq!(FieldValue(&Type::SignedInteger(83)).escape(), r#"83i"#);

        assert_eq!(FieldValue(&Type::Text("".into())).escape(), r#""""#);
        assert_eq!(FieldValue(&Type::Text("0".into())).escape(), r#""0""#);
        assert_eq!(FieldValue(&Type::Text("\"".into())).escape(), r#""\"""#);
        assert_eq!(
            FieldValue(&Type::Text(r#"locat"\ ,=ion"#.into())).escape(),
            r#""locat\"\\ ,=ion""#
        );
    }

    #[test]
    fn test_empty_tag_value() {
        // InfluxDB doesn't support empty tag values. But that's a job
        // of a calling site to validate an entire write request.
        assert_eq!(TagValue(&Type::Text("".into())).escape(), r#""#);
    }
}

/// InfluxDB Line Protocol escaping helper module.
/// https://docs.influxdata.com/influxdb/v1.7/write_protocols/line_protocol_tutorial/
use crate::Type;
use lazy_regex::{lazy_regex, Lazy, Regex};

pub static COMMAS_SPACES: Lazy<Regex> = lazy_regex!("[, ]");
pub static COMMAS_SPACES_EQUALS: Lazy<Regex> = lazy_regex!("[, =]");
pub static QUOTES_SLASHES: Lazy<Regex> = lazy_regex!(r#"["\\]"#);
pub static SLASHES: Lazy<Regex> = lazy_regex!(r#"(\\|,| |=|")"#);

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
            FieldValue(x) => Self::escape_field_value(x, false),
            TagValue(x) => Self::escape_tag_value(x),
        }
    }

    pub fn escape_v2(self) -> String {
        use LineProtoTerm::*;
        match self {
            Measurement(x) => Self::escape_any(x, &COMMAS_SPACES),
            TagKey(x) | FieldKey(x) => Self::escape_any(x, &COMMAS_SPACES_EQUALS),
            FieldValue(x) => Self::escape_field_value(x, true),
            TagValue(x) => Self::escape_tag_value(x),
        }
    }

    /// Serializes Field Values.
    fn escape_field_value(v: &Type, use_v2: bool) -> String {
        use Type::*;
        match v {
            Boolean(v) => Self::escape_boolean(v),
            Float(v) => Self::escape_float(v),
            SignedInteger(v) => Self::escape_signed_integer(v),
            UnsignedInteger(v) => Self::escape_unsigned_integer(v, use_v2),
            Text(v) => format!(r#""{}""#, Self::escape_any(v, &QUOTES_SLASHES)),
        }
    }

    /// Serializes Tag Values. InfluxDB stores tag values as strings, so we format everything to string.
    ///
    /// V2: https://docs.influxdata.com/influxdb/cloud/reference/syntax/line-protocol/#tag-set
    /// V1: https://docs.influxdata.com/influxdb/v1/write_protocols/line_protocol_tutorial/#data-types
    fn escape_tag_value(v: &Type) -> String {
        use Type::*;
        match v {
            Boolean(v) => Self::escape_boolean(v),
            Float(v) => format!(r#"{v}"#),
            SignedInteger(v) => format!(r#"{v}"#),
            UnsignedInteger(v) => format!(r#"{v}"#),
            Text(v) => Self::escape_any(v, &SLASHES),
        }
    }

    fn escape_any(s: &str, re: &Regex) -> String {
        re.replace_all(s, r"\$0").to_string()
    }

    /// Escapes a Rust f64 to InfluxDB Line Protocol
    ///
    /// https://docs.influxdata.com/influxdb/cloud/reference/syntax/line-protocol/#float
    ///     IEEE-754 64-bit floating-point numbers. Default numerical type.
    ///     InfluxDB supports scientific notation in float field values, but this crate does not serialize them.
    fn escape_float(v: &f64) -> String {
        v.to_string()
    }

    /// Escapes a Rust bool to InfluxDB Line Protocol
    ///
    /// https://docs.influxdata.com/influxdb/cloud/reference/syntax/line-protocol/#boolean
    ///     Stores true or false values.
    fn escape_boolean(v: &bool) -> String {
        if *v { "true" } else { "false" }.to_string()
    }

    /// Escapes a Rust i64 to InfluxDB Line Protocol
    ///
    /// https://docs.influxdata.com/influxdb/cloud/reference/syntax/line-protocol/#integer
    ///     Signed 64-bit integers. Trailing i on the number specifies an integer.
    fn escape_signed_integer(v: &i64) -> String {
        format!("{v}i")
    }

    /// Escapes a Rust u64 to InfluxDB Line Protocol
    ///
    /// https://docs.influxdata.com/influxdb/cloud/reference/syntax/line-protocol/#uinteger
    ///     Unsigned 64-bit integers. Trailing u on the number specifies an unsigned integer.
    ///
    /// InfluxDB version 1 does not know unsigned, we fallback to (signed) integer:
    /// https://docs.influxdata.com/influxdb/v1/write_protocols/line_protocol_tutorial/#data-types
    fn escape_unsigned_integer(v: &u64, use_v2: bool) -> String {
        if use_v2 {
            format!("{v}u")
        } else {
            format!("{v}i")
        }
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
            r"this\ is\ my\ special\ string"
        );
        assert_eq!(
            TagValue(&Type::Text("a tag w=i th == tons of escapes".into())).escape(),
            r"a\ tag\ w\=i\ th\ \=\=\ tons\ of\ escapes"
        );
        assert_eq!(
            TagValue(&Type::Text("no_escapes".into())).escape(),
            r#"no_escapes"#
        );
        assert_eq!(
            TagValue(&Type::Text("some,commas,here".into())).escape(),
            r"some\,commas\,here"
        );

        assert_eq!(Measurement(r#"wea", ther"#).escape(), r#"wea"\,\ ther"#);
        assert_eq!(TagKey(r"locat\ ,=ion").escape(), r"locat\\ \,\=ion");

        assert_eq!(FieldValue(&Type::Boolean(true)).escape(), r#"true"#);
        assert_eq!(FieldValue(&Type::Boolean(false)).escape(), r#"false"#);

        assert_eq!(FieldValue(&Type::Float(0.0)).escape(), r#"0"#);
        assert_eq!(FieldValue(&Type::Float(-0.1)).escape(), r#"-0.1"#);

        assert_eq!(FieldValue(&Type::SignedInteger(0)).escape(), r#"0i"#);
        assert_eq!(FieldValue(&Type::SignedInteger(83)).escape(), r#"83i"#);

        assert_eq!(FieldValue(&Type::UnsignedInteger(0)).escape(), r#"0i"#);
        assert_eq!(FieldValue(&Type::UnsignedInteger(83)).escape(), r#"83i"#);

        assert_eq!(FieldValue(&Type::UnsignedInteger(0)).escape_v2(), r#"0u"#);
        assert_eq!(FieldValue(&Type::UnsignedInteger(83)).escape_v2(), r#"83u"#);

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

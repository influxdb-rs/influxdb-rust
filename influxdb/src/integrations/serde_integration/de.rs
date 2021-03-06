use super::{Series, TaggedSeries};
use serde::de::{
    value, DeserializeSeed, Deserializer, Error, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};
use serde::Deserialize;
use std::fmt;
use std::marker::PhantomData;

// Based on https://serde.rs/deserialize-struct.html
impl<'de, T> Deserialize<'de> for Series<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Field name deserializer
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Name,
            Columns,
            Values,
        }

        struct SeriesVisitor<T> {
            _inner_type: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for SeriesVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Series<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Series")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Series<T>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut columns: Option<Vec<String>> = None;
                let mut values: Option<Vec<T>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Columns => {
                            if columns.is_some() {
                                return Err(Error::duplicate_field("columns"));
                            }
                            columns = Some(map.next_value()?);
                        }
                        Field::Values => {
                            if values.is_some() {
                                return Err(Error::duplicate_field("values"));
                            }
                            // Error out if "values" is encountered before "columns"
                            // Hopefully, InfluxDB never does this.
                            if columns.is_none() {
                                return Err(Error::custom(
                                    "series values encountered before columns",
                                ));
                            }
                            // Deserialize using a HeaderVec deserializer
                            // seeded with the headers from the "columns" field
                            values = Some(map.next_value_seed(HeaderVec::<T> {
                                header: columns.as_ref().unwrap(),
                                _inner_type: PhantomData,
                            })?);
                        }
                    }
                }
                let name = name.ok_or_else(|| Error::missing_field("name"))?;
                let values = values.unwrap_or_default();

                Ok(Series { name, values })
            }
        }

        const FIELDS: &[&str] = &["name", "values"];
        deserializer.deserialize_struct(
            "Series",
            FIELDS,
            SeriesVisitor::<T> {
                _inner_type: PhantomData,
            },
        )
    }
}

// Based on https://serde.rs/deserialize-struct.html
impl<'de, TAG, T> Deserialize<'de> for TaggedSeries<TAG, T>
where
    TAG: Deserialize<'de>,
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Field name deserializer
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Name,
            Tags,
            Columns,
            Values,
        }

        struct SeriesVisitor<TAG, T> {
            _tag_type: PhantomData<TAG>,
            _value_type: PhantomData<T>,
        }

        impl<'de, TAG, T> Visitor<'de> for SeriesVisitor<TAG, T>
        where
            TAG: Deserialize<'de>,
            T: Deserialize<'de>,
        {
            type Value = TaggedSeries<TAG, T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct TaggedSeries")
            }

            fn visit_map<V>(self, mut map: V) -> Result<TaggedSeries<TAG, T>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut tags: Option<TAG> = None;
                let mut columns: Option<Vec<String>> = None;
                let mut values: Option<Vec<T>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Tags => {
                            if tags.is_some() {
                                return Err(Error::duplicate_field("tags"));
                            }
                            tags = Some(map.next_value()?);
                        }
                        Field::Columns => {
                            if columns.is_some() {
                                return Err(Error::duplicate_field("columns"));
                            }
                            columns = Some(map.next_value()?);
                        }
                        Field::Values => {
                            if values.is_some() {
                                return Err(Error::duplicate_field("values"));
                            }
                            // Error out if "values" is encountered before "columns"
                            // Hopefully, InfluxDB never does this.
                            if columns.is_none() {
                                return Err(Error::custom(
                                    "series values encountered before columns",
                                ));
                            }
                            // Deserialize using a HeaderVec deserializer
                            // seeded with the headers from the "columns" field
                            values = Some(map.next_value_seed(HeaderVec::<T> {
                                header: columns.as_ref().unwrap(),
                                _inner_type: PhantomData,
                            })?);
                        }
                    }
                }
                let name = name.ok_or_else(|| Error::missing_field("name"))?;
                let tags = tags.ok_or_else(|| Error::missing_field("tags"))?;
                let values = values.ok_or_else(|| Error::missing_field("values"))?;
                Ok(TaggedSeries { name, tags, values })
            }
        }

        const FIELDS: &[&str] = &["name", "tags", "values"];
        deserializer.deserialize_struct(
            "TaggedSeries",
            FIELDS,
            SeriesVisitor::<TAG, T> {
                _tag_type: PhantomData,
                _value_type: PhantomData,
            },
        )
    }
}

// Deserializer that takes a header as a seed
// and deserializes an array of arrays into a
// Vec of map-like values using the header as
// keys and the values as values.
struct HeaderVec<'h, T> {
    header: &'h [String],
    _inner_type: PhantomData<T>,
}

impl<'de, 'h, T> DeserializeSeed<'de> for HeaderVec<'h, T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HeaderVecVisitor<'h, T> {
            header: &'h [String],
            _inner_type: PhantomData<T>,
        }
        impl<'de, 'h, T> Visitor<'de> for HeaderVecVisitor<'h, T>
        where
            T: Deserialize<'de>,
        {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of arrays")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(v) = seq.next_element_seed(RowWithHeader {
                    header: self.header,
                    _inner_type: PhantomData,
                })? {
                    vec.push(v);
                }

                Ok(vec)
            }
        }
        deserializer.deserialize_seq(HeaderVecVisitor {
            header: self.header,
            _inner_type: PhantomData,
        })
    }
}

// Deserializer that takes a header as a seed
// and deserializes an array into a map-like
// value using the header as keys and the values
// as values.
struct RowWithHeader<'h, T> {
    header: &'h [String],
    _inner_type: PhantomData<T>,
}

impl<'de, 'h, T> DeserializeSeed<'de> for RowWithHeader<'h, T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RowWithHeaderVisitor<'h, T> {
            header: &'h [String],
            _inner: PhantomData<fn() -> T>,
        }

        impl<'de, 'h, T> Visitor<'de> for RowWithHeaderVisitor<'h, T>
        where
            T: Deserialize<'de>,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("array")
            }

            fn visit_seq<A>(self, seq: A) -> Result<T, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
                // into a `Deserializer`, allowing it to be used as the input to T's
                // `Deserialize` implementation. T then deserializes itself using
                // the entries from the map visitor.
                Deserialize::deserialize(value::MapAccessDeserializer::new(HeaderMapAccess {
                    header: self.header,
                    field: 0,
                    data: seq,
                }))
            }
        }

        deserializer.deserialize_seq(RowWithHeaderVisitor {
            header: self.header,
            _inner: PhantomData,
        })
    }
}

// MapAccess implementation that holds a reference to
// the header for keys and a serde sequence for values.
// When asked for a key, it returns the next header and
// advances its header field index. When asked for a value,
// it tries to deserialize the next element in the serde
// sequence into the desired type, and returns an error
// if no element is returned (the sequence is exhausted).
struct HeaderMapAccess<'h, A> {
    header: &'h [String],
    field: usize,
    data: A,
}

impl<'de, 'h, A> MapAccess<'de> for HeaderMapAccess<'h, A>
where
    A: SeqAccess<'de>,
{
    type Error = <A as SeqAccess<'de>>::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        let field = match self.header.get(self.field) {
            None => return Ok(None),
            Some(field) => field,
        };
        self.field += 1;
        seed.deserialize(field.clone().into_deserializer())
            .map(Some)
    }

    fn next_value_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<K::Value, Self::Error> {
        match self.data.next_element_seed(seed)? {
            Some(value) => Ok(value),
            None => Err(Error::custom("next_value_seed called but no value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Series;
    use std::borrow::Cow;
    use std::collections::HashMap;

    const TEST_DATA: &str = r#"
    {
        "name": "series_name",
        "columns": ["foo", "bar"],
        "values": [
            ["foo_a", "bar_a"],
            ["foo_b", "bar_b"]
        ]
    }
    "#;

    // we can derive all the impls we want here
    #[derive(Debug, PartialEq, Eq)]
    struct EqSeries<T> {
        pub name: String,
        pub values: Vec<T>,
    }

    impl<T> From<Series<T>> for EqSeries<T> {
        fn from(Series { name, values }: Series<T>) -> Self {
            EqSeries { name, values }
        }
    }

    #[test]
    fn test_deserialize_cow() {
        // Unfortunately, Cow is not automatically borrowed,
        // so this is basically equivalent to String, String
        let result = serde_json::from_str::<Series<HashMap<Cow<str>, Cow<str>>>>(TEST_DATA);
        assert!(result.is_ok());
        assert_eq!(
            EqSeries::from(result.unwrap()),
            EqSeries {
                name: "series_name".into(),
                values: vec![
                    {
                        let mut h = std::collections::HashMap::new();
                        h.insert("foo".into(), "foo_a".into());
                        h.insert("bar".into(), "bar_a".into());
                        h
                    },
                    {
                        let mut h = std::collections::HashMap::new();
                        h.insert("foo".into(), "foo_b".into());
                        h.insert("bar".into(), "bar_b".into());
                        h
                    },
                ],
            },
        );
    }

    #[test]
    fn test_deserialize_borrowed() {
        use serde::Deserialize;

        // Deserializing a string that cannot be passed through
        // without escaping will result in an error like this:
        // `invalid type: string "\n", expected a borrowed string at line 6 column 43`
        // but if it doesn't need escaping it's fine.
        #[derive(Deserialize, Debug, PartialEq, Eq)]
        struct BorrowingStruct<'a> {
            foo: &'a str,
            bar: &'a str,
        }

        let result = serde_json::from_str::<Series<BorrowingStruct>>(TEST_DATA);
        assert!(result.is_ok(), "{}", result.unwrap_err());
        assert_eq!(
            EqSeries::from(result.unwrap()),
            EqSeries {
                name: "series_name".into(),
                values: vec![
                    BorrowingStruct {
                        foo: "foo_a",
                        bar: "bar_a",
                    },
                    BorrowingStruct {
                        foo: "foo_b",
                        bar: "bar_b",
                    },
                ],
            },
        );
    }
}

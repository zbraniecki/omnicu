// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

mod error;
mod parser;

use crate::fields::{self, Field, FieldLength, FieldSymbol};
pub use error::Error;
use parser::Parser;
use std::{
    convert::TryFrom,
    fmt::{self, Write},
    borrow::Cow,
    iter::FromIterator,
};

#[cfg(feature = "provider_serde")]
use serde::{
    de,
    ser::{self, SerializeSeq},
    Deserialize, Deserializer, Serialize,
};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum PatternItem<'s> {
    Field(fields::Field),
    #[serde(borrow)]
    Literal(Cow<'s, str>),
}

impl<'s> From<(FieldSymbol, FieldLength)> for PatternItem<'s> {
    fn from(input: (FieldSymbol, FieldLength)) -> Self {
        Self::Field(Field {
            symbol: input.0,
            length: input.1,
        })
    }
}

impl<'s> TryFrom<(FieldSymbol, u8)> for PatternItem<'s> {
    type Error = Error;
    fn try_from(input: (FieldSymbol, u8)) -> Result<Self, Self::Error> {
        let length = FieldLength::try_from(input.1).map_err(|_| Error::FieldTooLong(input.0))?;
        Ok(Self::Field(Field {
            symbol: input.0,
            length,
        }))
    }
}

impl<'s> From<&'s str> for PatternItem<'s> {
    fn from(input: &'s str) -> Self {
        Self::Literal(input.into())
    }
}

impl<'s> From<String> for PatternItem<'s> {
    fn from(input: String) -> Self {
        Self::Literal(input.into())
    }
}

/// The granularity of time represented in a pattern item.
/// Ordered from least granular to most granular for comparsion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) enum TimeGranularity {
    Hours,
    Minutes,
    Seconds,
}

#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Pattern<'s> {
    #[serde(borrow)]
    items: Vec<PatternItem<'s>>,
    #[cfg_attr(
        all(feature="provider_serde", not(feature="serialize_none")),
        serde(skip_serializing_if = "Option::is_none"))
    ]
    time_granularity: Option<TimeGranularity>,
}

/// Retrieves the granularity of time represented by a `PatternItem`.
/// If the `PatternItem` is not time-related, returns `None`.
fn get_time_granularity(item: &PatternItem) -> Option<TimeGranularity> {
    match item {
        PatternItem::Field(field) => match field.symbol {
            fields::FieldSymbol::Hour(_) => Some(TimeGranularity::Hours),
            fields::FieldSymbol::Minute => Some(TimeGranularity::Minutes),
            fields::FieldSymbol::Second(_) => Some(TimeGranularity::Seconds),
            _ => None,
        },
        _ => None,
    }
}

impl<'s> Pattern<'s> {
    pub fn items(&self) -> &[PatternItem] {
        &self.items
    }

    pub fn from_bytes(input: &'s str) -> Result<Self, Error> {
        Parser::new(input).parse().map(Pattern::from)
    }

    // TODO(#277): This should be turned into a utility for all ICU4X.
    pub fn from_bytes_combination(input: &str, date: Self, time: Self) -> Result<Self, Error> {
        panic!();
        // Parser::new(input)
        //     .parse_placeholders(vec![time, date])
        //     .map(Pattern::from)
    }

    pub(super) fn most_granular_time(&self) -> Option<TimeGranularity> {
        self.time_granularity
    }
}

impl<'s> From<Vec<PatternItem<'s>>> for Pattern<'s> {
    fn from(items: Vec<PatternItem<'s>>) -> Self {
        Self {
            time_granularity: items.iter().filter_map(get_time_granularity).max(),
            items,
        }
    }
}

/// This trait is implemented in order to provide the machinery to convert a `Pattern` to a UTS 35
/// pattern string. It could also be implemented as the Writeable trait, but at the time of writing
/// this was not done, as this code would need to implement the `write_len` method, which would
/// need to duplicate the branching logic of the `fmt` method here. This code is used in generating
/// the data providers and is not as performance sensitive.
impl<'s> fmt::Display for Pattern<'s> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for pattern_item in self.items().iter() {
            match pattern_item {
                PatternItem::Field(field) => {
                    let ch: char = field.symbol.into();
                    for _ in 0..field.length as usize {
                        formatter.write_char(ch)?;
                    }
                }
                PatternItem::Literal(literal) => {
                    // Determine if the literal contains any characters that would need to be escaped.
                    let mut needs_escaping = false;
                    for ch in literal.chars() {
                        if ch.is_ascii_alphabetic() || ch == '\'' {
                            needs_escaping = true;
                            break;
                        }
                    }

                    if needs_escaping {
                        let mut ch_iter = literal.trim_end().chars().peekable();

                        // Do not escape the leading whitespace.
                        while let Some(ch) = ch_iter.peek() {
                            if ch.is_whitespace() {
                                formatter.write_char(*ch)?;
                                ch_iter.next();
                            } else {
                                break;
                            }
                        }

                        // Wrap in "'" and escape "'".
                        formatter.write_char('\'')?;
                        for ch in ch_iter {
                            if ch == '\'' {
                                // Escape a single quote.
                                formatter.write_char('\\')?;
                            }
                            formatter.write_char(ch)?;
                        }
                        formatter.write_char('\'')?;

                        // Add the trailing whitespace
                        for ch in literal.chars().rev() {
                            if ch.is_whitespace() {
                                formatter.write_char(ch)?;
                            } else {
                                break;
                            }
                        }
                    } else {
                        formatter.write_str(literal)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'s> FromIterator<PatternItem<'s>> for Pattern<'s> {
    fn from_iter<I: IntoIterator<Item = PatternItem<'s>>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

// #[cfg(feature = "provider_serde")]
// struct DeserializePatternUTS35String;

// #[cfg(feature = "provider_serde")]
// impl<'de> de::Visitor<'de> for DeserializePatternUTS35String {
//     type Value = Pattern<'de>;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         write!(formatter, "Expected to find a valid pattern.")
//     }

//     fn visit_str<E>(self, pattern_string: &str) -> Result<Self::Value, E>
//     where
//         E: de::Error,
//     {
//         // Parse a string into a list of fields.
//         // Pattern::from_bytes(pattern_string).map_err(|err| {
//         //     de::Error::invalid_value(
//         //         de::Unexpected::Other(&format!("{}", err)),
//         //         &"a valid UTS 35 pattern string",
//         //     )
//         // })
//         panic!()
//     }
// }

// #[cfg(feature = "provider_serde")]
// struct DeserializePatternBincode;

// #[cfg(feature = "provider_serde")]
// impl<'de> de::Visitor<'de> for DeserializePatternBincode {
//     type Value = Pattern<'de>;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         write!(formatter, "Unable to deserialize a bincode Pattern.")
//     }

//     fn visit_seq<V>(self, mut seq: V) -> Result<Pattern<'de>, V::Error>
//     where
//         V: de::SeqAccess<'de>,
//     {
//         let mut items = Vec::new();
//         while let Some(item) = seq.next_element()? {
//             items.push(item)
//         }
//         Ok(Pattern::from(items))
//     }
// }

// #[cfg(feature = "provider_serde")]
// impl<'de> Deserialize<'de> for Pattern<'de> {
//     fn deserialize<D>(deserializer: D) -> Result<Pattern<'de>, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         if deserializer.is_human_readable() {
//             deserializer.deserialize_str(DeserializePatternUTS35String)
//         } else {
//             deserializer.deserialize_seq(DeserializePatternBincode)
//         }
//     }
// }

// #[cfg(feature = "provider_serde")]
// impl<'s> Serialize for Pattern<'s> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: ser::Serializer,
//     {
//         if serializer.is_human_readable() {
//             // Serialize into the UTS 35 string representation.
//             let string: String = self.to_string();
//             serializer.serialize_str(&string)
//         } else {
//             // Serialize into a bincode-friendly representation. This means that pattern parsing
//             // will not be needed when deserializing.
//             let mut seq = serializer.serialize_seq(Some(self.items.len()))?;
//             for item in self.items.iter() {
//                 seq.serialize_element(item)?;
//             }
//             seq.end()
//         }
//     }
// }

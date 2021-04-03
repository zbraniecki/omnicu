// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! `icu_pattern` is a utility crate of the [`ICU4X`] project.
//!
//! It includes a [`Parser`]/[`Interpolator`] pair. The pair can be
//! used to parse and interpolate ICU placeholder patterns, like "{0} days" or
//! "{0}, {1}" with custom elements and string literals.
//!
//! # Placeholders & Elements
//!
//! An `Element` may be any type that implements [`From<&str>`][`From`].
//! A `Placeholder` may be any type that implements [`FromStr`].
//!
//! # Examples
//!
//! In the following example we're going to use a custom `Token` type,
//! and an `Element` type which will be either a `Token` or a string slice.
//!
//! ```
//! use icu_pattern::{
//!     Parser,
//!     ParserOptions,
//!     PatternToken,
//!     Interpolator,
//!     InterpolatedKind,
//! };
//! use std::{
//!     convert::TryInto,
//!     borrow::Cow
//! };
//!
//! #[derive(Debug, PartialEq)]
//! enum ExampleToken {
//!     Variant1,
//!     Variant2
//! }
//!
//! #[derive(Debug, PartialEq)]
//! enum ExampleElement<'s> {
//!     Token(ExampleToken),
//!     Literal(Cow<'s, str>),
//! }
//!
//! impl<'s> From<Cow<'s, str>> for ExampleElement<'s> {
//!     fn from(input: Cow<'s, str>) -> Self {
//!         Self::Literal(input)
//!     }
//! }
//!
//! let pattern: Vec<PatternToken<'_, usize>> = Parser::new("{0}, {1}", ParserOptions {
//!    allow_raw_letters: false
//! }).try_into().unwrap();
//!
//! let replacements = vec![
//!     vec![
//!         ExampleElement::Token(ExampleToken::Variant1),
//!         ExampleElement::Literal(" foo ".into()),
//!         ExampleElement::Token(ExampleToken::Variant2),
//!     ],
//!     vec![
//!         ExampleElement::Token(ExampleToken::Variant2),
//!         ExampleElement::Literal(" bar ".into()),
//!         ExampleElement::Token(ExampleToken::Variant1),
//!     ],
//! ];
//!
//! let mut interpolator = Interpolator::<Vec<Vec<_>>, ExampleElement>::new(&pattern, &replacements);
//!
//! let mut result = vec![];
//!
//! while let Some(ik) = interpolator.try_next().expect("Failed to advance iterator") {
//!     result.push(ik);
//! }
//!
//! assert_eq!(result, &[
//!     InterpolatedKind::Element(&ExampleElement::Token(ExampleToken::Variant1)),
//!     InterpolatedKind::Element(&ExampleElement::Literal(" foo ".into())),
//!     InterpolatedKind::Element(&ExampleElement::Token(ExampleToken::Variant2)),
//!     InterpolatedKind::Literal(&", ".into()),
//!     InterpolatedKind::Element(&ExampleElement::Token(ExampleToken::Variant2)),
//!     InterpolatedKind::Element(&ExampleElement::Literal(" bar ".into())),
//!     InterpolatedKind::Element(&ExampleElement::Token(ExampleToken::Variant1)),
//! ]);
//! ```
//!
//! # Combinators
//!
//! In the example above, the replacements will be parsed at compile time and stored on a [`Vec`],
//! which is a collection type that has an implementation for [`ReplacementProvider`]
//! trait.
//!
//! In real use, the consumer may want to use different models of replacement provider,
//! and different element schemas.
//! Because the replacement is an iterator itself, it allows for other, more specialized parsers,
//! to be used to lazily parse particular patterns that are meant to replace the placeholders.
//! This allows for lazy parsing of those specialized patterns to be triggered
//! only if the placeholder pattern encounters a placeholder key that requires given
//! pattern to be used.
//!
//! [`ICU4X`]: ../icu/index.html
//! [`FromStr`]: std::str::FromStr
mod interpolator;
mod parser;
mod replacement;
mod token;

pub use interpolator::{InterpolatedKind, Interpolator, InterpolatorError};
pub use parser::{Parser, ParserError, ParserOptions};
pub use replacement::ReplacementProvider;
pub use token::PatternToken;

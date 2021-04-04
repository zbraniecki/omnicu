// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

mod error;

use crate::parser::error::ParserError;
use crate::token::PatternToken;
use crate::{
    interpolator::{InterpolatedKind, Interpolator, InterpolatorError},
    parser::{Parser, ParserOptions},
    replacement::ReplacementProvider,
};
pub use error::PatternError;
use std::{
    convert::{TryFrom, TryInto},
    fmt::{Debug, Display, Write},
    str::FromStr,
};

/// `Pattern` stores the result of parsing operation as a vector
/// of [`PatternToken`] elements.
///
/// # Type parameters
///
/// - `P`: The type of the placeholder used as a key for the [`ReplacementProvider`].
///
/// # Lifetimes
///
/// - `p`: The life time of an input string slice to be parsed.
///
/// [`ReplacementProvider`]: crate::ReplacementProvider
#[derive(Debug)]
pub struct Pattern<'s, P>(pub(crate) Vec<PatternToken<'s, P>>);

impl<'s, P> Pattern<'s, P> {
    /// Interpolates the `Pattern` with provided replacements and returns
    /// a [`InterpolatedPattern`] structure.
    ///
    /// For allocation-free interpolation, see `interpolate_to_string` or
    /// `interpolate_to_write`.
    ///
    /// For lower level interpolation iterator see [`Interpolator`].
    pub fn interpolate<'i, E, R>(
        &'i self,
        replacements: &'i R,
    ) -> Result<InterpolatedPattern<'i, 's, E>, PatternError<R::Key>>
    where
        R: ReplacementProvider<'i, E, Key = P>,
        P: Debug + FromStr + Clone,
        <P as FromStr>::Err: Debug,
    {
        Interpolator::new(self, replacements)
            .try_into()
            .map_err(Into::into)
    }

    /// Interpolates the `Pattern` with provided replacements and a new
    /// [`String`].
    ///
    /// For buffer write interpolation, see `interpolate_to_write`.
    ///
    /// For lower level interpolation iterator see [`Interpolator`].
    pub fn interpolate_to_string<'i, E, R>(
        &'i self,
        replacements: &'i R,
    ) -> Result<String, PatternError<R::Key>>
    where
        R: ReplacementProvider<'i, E, Key = P>,
        P: Debug + FromStr + Clone,
        <P as FromStr>::Err: Debug,
        E: 'i + Display,
    {
        let mut result = String::new();
        self.interpolate_to_write(replacements, &mut result)?;
        Ok(result)
    }

    /// Interpolates the `Pattern` writing the result into a buffer.
    pub fn interpolate_to_write<'i, E, R, W>(
        &'i self,
        replacements: &'i R,
        f: &mut W,
    ) -> Result<(), PatternError<R::Key>>
    where
        R: ReplacementProvider<'i, E, Key = P>,
        P: Debug + FromStr + Clone,
        <P as FromStr>::Err: Debug,
        E: 'i + Display,
        W: Write,
    {
        let mut interpolator = Interpolator::new(self, replacements);
        while let Some(ik) = interpolator.try_next()? {
            write!(f, "{}", ik)?;
        }
        Ok(())
    }
}

impl<'s, P> TryFrom<&'s str> for Pattern<'s, P>
where
    P: FromStr,
    <P as FromStr>::Err: Debug,
{
    type Error = ParserError<<P as FromStr>::Err>;

    fn try_from(input: &'s str) -> Result<Self, Self::Error> {
        Ok(Parser::new(
            input,
            ParserOptions {
                allow_raw_letters: false,
            },
        )
        .try_into()?)
    }
}

impl<'p, P> TryFrom<Parser<'p, P>> for Pattern<'p, P>
where
    P: FromStr,
    <P as FromStr>::Err: Debug,
{
    type Error = ParserError<<P as FromStr>::Err>;

    fn try_from(mut parser: Parser<'p, P>) -> Result<Self, Self::Error> {
        let mut result = vec![];
        while let Some(token) = parser.try_next()? {
            result.push(token);
        }
        Ok(Self(result))
    }
}

impl<'p, P> From<Vec<PatternToken<'p, P>>> for Pattern<'p, P>
where
    P: FromStr,
    <P as FromStr>::Err: Debug,
{
    fn from(tokens: Vec<PatternToken<'p, P>>) -> Self {
        Self(tokens)
    }
}

/// `InterpolatedPattern` stores the result of parsing operation as a vector
/// of [`InterpolatedKind`] elements.
///
/// # Type parameters
///
/// - `E`: An element type returned by the `ReplacementProvider`.
///
/// # Lifetimes
///
/// - `i`: The life time of `ReplacementProvider`.
/// - `s`: The life time of literals stored in the `E`
///
/// [`ReplacementProvider`]: crate::ReplacementProvider
pub struct InterpolatedPattern<'i, 's, E>(Vec<InterpolatedKind<'i, 's, E>>);

impl<'i, 's, R, E> TryFrom<Interpolator<'i, 's, R, E>> for InterpolatedPattern<'i, 's, E>
where
    R: ReplacementProvider<'i, E>,
    R::Key: FromStr + Clone + Debug,
    <R::Key as FromStr>::Err: Debug,
{
    type Error = InterpolatorError<R::Key>;

    fn try_from(mut interpolator: Interpolator<'i, 's, R, E>) -> Result<Self, Self::Error> {
        let mut result = vec![];
        while let Some(ik) = interpolator.try_next()? {
            result.push(ik);
        }
        Ok(Self(result))
    }
}

impl<'i, 's, E> Display for InterpolatedPattern<'i, 's, E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for elem in &self.0 {
            write!(f, "{}", elem)?;
        }
        Ok(())
    }
}

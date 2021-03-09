use icu_pattern::*;
use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Write;

#[derive(Debug)]
struct Token;

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn iai_parse() {
    let samples = vec![
        ("Foo {0} and {1}", vec![vec!["Hello"], vec!["World"]]),
        ("Foo {1} and {0}", vec![vec!["Hello"], vec!["World"]]),
        (
            "{0}, {1} and {2}",
            vec![vec!["Start"], vec!["Middle"], vec!["End"]],
        ),
        // ("{start}, {midde} and {end}", vec!["Start", "Middle", "End"]),
        ("{0} 'at' {1}", vec![vec!["Hello"], vec!["World"]]),
    ];

    for sample in &samples {
        let _ = Parser::<_, usize>::new(&sample.0).count();
    }
}

pub enum Element<'s> {
    Literal(Cow<'s, str>),
}

impl Display for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(s) => f.write_str(s),
        }
    }
}

impl<'s> From<Cow<'s, str>> for Element<'s> {
    fn from(input: Cow<'s, str>) -> Self {
        Self::Literal(input)
    }
}

impl<'s> From<&'s str> for Element<'s> {
    fn from(input: &'s str) -> Self {
        Self::Literal(input.into())
    }
}

fn iai_interpolate() {
    let samples = vec![
        ("Foo {0} and {1}", vec![vec!["Hello"], vec!["World"]]),
        ("Foo {1} and {0}", vec![vec!["Hello"], vec!["World"]]),
        (
            "{0}, {1} and {2}",
            vec![vec!["Start"], vec!["Middle"], vec!["End"]],
        ),
        ("{0} 'at' {1}", vec![vec!["Hello"], vec!["World"]]),
    ];

    for sample in &samples {
        let iter = Parser::<_, usize>::new(&sample.0);

        let replacements: Vec<Vec<Element>> = sample
            .1
            .iter()
            .map(|r| r.iter().map(|&t| t.into()).collect())
            .collect();

        let mut i = interpolate(iter, replacements);
        let _ = i
            .try_fold(String::new(), |mut acc, t| {
                if t.map(|t| write!(acc, "{}", t)).is_err() {
                    Err(())
                } else {
                    Ok(acc)
                }
            })
            .unwrap();
    }
}

fn iai_named_interpolate() {
    let named_samples = vec![(
        "{start}, {midde} and {end}",
        vec![
            ("start", vec!["Start"]),
            ("middle", vec!["Middle"]),
            ("end", vec!["End"]),
        ],
    )];

    for sample in &named_samples {
        let iter = Parser::<_, String>::new(&sample.0);

        let replacements: std::collections::HashMap<String, Vec<Element>> = sample
            .1
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|&t| t.into()).collect()))
            .collect();

        let _ = interpolate(iter, replacements).count();
    }
}

iai::main!(iai_parse, iai_interpolate, iai_named_interpolate);

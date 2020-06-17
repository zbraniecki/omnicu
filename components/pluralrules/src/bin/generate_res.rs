use bincode;
use icu_pluralrules::data::json::get_resource;
use icu_pluralrules::PluralRuleType;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    {
        let res = get_resource(PluralRuleType::Cardinal);
        let encoded: Vec<u8> = bincode::serialize(&res).unwrap();
        let mut buffer = File::create("./data/plurals.dat").expect("Opening file failed");
        buffer.write_all(&encoded).expect("Writing failed");
    }
    {
        let res = get_resource(PluralRuleType::Ordinal);
        let encoded: Vec<u8> = bincode::serialize(&res).unwrap();
        let mut buffer = File::create("./data/ordinals.dat").expect("Opening file failed");
        buffer.write_all(&encoded).expect("Writing failed");
    }
}
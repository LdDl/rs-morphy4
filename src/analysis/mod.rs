mod analyzer;
mod base;
mod compound;
mod dictionary;
mod dictionary_parse;
mod normalization;
mod parse;
mod patterns;
mod prediction;
mod special;
mod superlative;
mod word;

pub use dictionary::DictionaryAnalyzer;
pub use parse::Parse;

pub use analyzer::MorphAnalyzer;

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// REVERSE(string) - Reverses a string
pub struct ReverseFunction;

impl SqlFunction for ReverseFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "REVERSE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Reverses the characters in a string",
            returns: "Reversed string",
            examples: vec![
                "SELECT REVERSE('hello') -- returns 'olleh'",
                "SELECT REVERSE('SQL CLI') -- returns 'ILC LQS'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("REVERSE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(s.chars().rev().collect())),
            DataValue::InternedString(s) => Ok(DataValue::String(s.chars().rev().collect())),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("REVERSE requires a string argument")),
        }
    }
}

/// INITCAP(string) / PROPER(string) - Capitalizes the first letter of each word
pub struct InitCapFunction;

impl SqlFunction for InitCapFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "INITCAP",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Capitalizes the first letter of each word",
            returns: "String with each word capitalized",
            examples: vec![
                "SELECT INITCAP('hello world') -- returns 'Hello World'",
                "SELECT INITCAP('sql-cli is great') -- returns 'Sql-Cli Is Great'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("INITCAP requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => {
                let mut result = String::new();
                let mut capitalize_next = true;

                for ch in s.chars() {
                    if ch.is_alphabetic() {
                        if capitalize_next {
                            result.push(ch.to_uppercase().next().unwrap_or(ch));
                            capitalize_next = false;
                        } else {
                            result.push(ch.to_lowercase().next().unwrap_or(ch));
                        }
                    } else {
                        result.push(ch);
                        capitalize_next = !ch.is_ascii_alphanumeric();
                    }
                }

                Ok(DataValue::String(result))
            }
            DataValue::InternedString(s) => {
                let mut result = String::new();
                let mut capitalize_next = true;

                for ch in s.chars() {
                    if ch.is_alphabetic() {
                        if capitalize_next {
                            result.push(ch.to_uppercase().next().unwrap_or(ch));
                            capitalize_next = false;
                        } else {
                            result.push(ch.to_lowercase().next().unwrap_or(ch));
                        }
                    } else {
                        result.push(ch);
                        capitalize_next = !ch.is_ascii_alphanumeric();
                    }
                }

                Ok(DataValue::String(result))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("INITCAP requires a string argument")),
        }
    }
}

/// PROPER(string) - Alias for INITCAP
pub struct ProperFunction;

impl SqlFunction for ProperFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PROPER",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Alias for INITCAP - capitalizes first letter of each word",
            returns: "String with each word capitalized",
            examples: vec!["SELECT PROPER('hello world') -- returns 'Hello World'"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        InitCapFunction.evaluate(args)
    }
}

/// ROT13(string) - ROT13 encoding
pub struct Rot13Function;

impl SqlFunction for Rot13Function {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ROT13",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Applies ROT13 encoding (shifts letters by 13 positions)",
            returns: "ROT13 encoded string",
            examples: vec![
                "SELECT ROT13('hello') -- returns 'uryyb'",
                "SELECT ROT13('uryyb') -- returns 'hello' (ROT13 is reversible)",
                "SELECT ROT13('SQL123') -- returns 'FDY123' (numbers unchanged)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("ROT13 requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => {
                let result: String = s
                    .chars()
                    .map(|ch| match ch {
                        'A'..='M' | 'a'..='m' => ((ch as u8) + 13) as char,
                        'N'..='Z' | 'n'..='z' => ((ch as u8) - 13) as char,
                        _ => ch,
                    })
                    .collect();
                Ok(DataValue::String(result))
            }
            DataValue::InternedString(s) => {
                let result: String = s
                    .chars()
                    .map(|ch| match ch {
                        'A'..='M' | 'a'..='m' => ((ch as u8) + 13) as char,
                        'N'..='Z' | 'n'..='z' => ((ch as u8) - 13) as char,
                        _ => ch,
                    })
                    .collect();
                Ok(DataValue::String(result))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("ROT13 requires a string argument")),
        }
    }
}

/// SOUNDEX(string) - Soundex phonetic encoding
pub struct SoundexFunction;

impl SqlFunction for SoundexFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SOUNDEX",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the Soundex code for phonetic matching",
            returns: "4-character Soundex code",
            examples: vec![
                "SELECT SOUNDEX('Smith') -- returns 'S530'",
                "SELECT SOUNDEX('Smythe') -- returns 'S530' (sounds similar)",
                "SELECT SOUNDEX('Johnson') -- returns 'J525'",
                "SELECT SOUNDEX('Jonson') -- returns 'J525' (sounds similar)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("SOUNDEX requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(compute_soundex(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(compute_soundex(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("SOUNDEX requires a string argument")),
        }
    }
}

fn compute_soundex(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    let s_upper = s.to_uppercase();
    let mut result = String::new();

    // Keep the first letter
    if let Some(first_char) = s_upper.chars().next() {
        if first_char.is_alphabetic() {
            result.push(first_char);
        } else {
            return String::new();
        }
    }

    // Soundex digit mapping
    let soundex_map: HashMap<char, char> = [
        ('B', '1'),
        ('F', '1'),
        ('P', '1'),
        ('V', '1'),
        ('C', '2'),
        ('G', '2'),
        ('J', '2'),
        ('K', '2'),
        ('Q', '2'),
        ('S', '2'),
        ('X', '2'),
        ('Z', '2'),
        ('D', '3'),
        ('T', '3'),
        ('L', '4'),
        ('M', '5'),
        ('N', '5'),
        ('R', '6'),
    ]
    .iter()
    .cloned()
    .collect();

    let mut last_code = ' ';
    for ch in s_upper.chars().skip(1) {
        if let Some(&code) = soundex_map.get(&ch) {
            if code != last_code {
                result.push(code);
                last_code = code;
            }
        } else if "AEIOUYHW".contains(ch) {
            last_code = ' ';
        }

        if result.len() >= 4 {
            break;
        }
    }

    // Pad with zeros to length 4
    while result.len() < 4 {
        result.push('0');
    }

    result.chars().take(4).collect()
}

/// PIG_LATIN(string) - Convert to Pig Latin
pub struct PigLatinFunction;

impl SqlFunction for PigLatinFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PIG_LATIN",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts text to Pig Latin",
            returns: "Pig Latin version of the text",
            examples: vec![
                "SELECT PIG_LATIN('hello') -- returns 'ellohay'",
                "SELECT PIG_LATIN('apple') -- returns 'appleway'",
                "SELECT PIG_LATIN('SQL') -- returns 'SQLay'",
                "SELECT PIG_LATIN('hello world') -- returns 'ellohay orldway'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("PIG_LATIN requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => {
                let result: Vec<String> = s
                    .split_whitespace()
                    .map(|word| pig_latin_word(word))
                    .collect();
                Ok(DataValue::String(result.join(" ")))
            }
            DataValue::InternedString(s) => {
                let result: Vec<String> = s
                    .split_whitespace()
                    .map(|word| pig_latin_word(word))
                    .collect();
                Ok(DataValue::String(result.join(" ")))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("PIG_LATIN requires a string argument")),
        }
    }
}

fn pig_latin_word(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let vowels = "aeiouAEIOU";
    let word_lower = word.to_lowercase();

    // Check if word starts with a vowel
    if let Some(first_char) = word_lower.chars().next() {
        if vowels.contains(first_char) {
            return format!("{}way", word);
        }
    }

    // Find the first vowel position
    if let Some(vowel_pos) = word_lower.chars().position(|c| vowels.contains(c)) {
        if vowel_pos > 0 {
            let consonant_cluster = &word[..vowel_pos];
            let rest = &word[vowel_pos..];
            return format!("{}{}ay", rest, consonant_cluster.to_lowercase());
        }
    }

    // No vowels found, just add 'ay'
    format!("{}ay", word.to_lowercase())
}

/// MORSE_CODE(string) - Convert to Morse code
pub struct MorseCodeFunction;

impl SqlFunction for MorseCodeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MORSE_CODE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Converts text to Morse code",
            returns: "Morse code representation",
            examples: vec![
                "SELECT MORSE_CODE('SOS') -- returns '... --- ...'",
                "SELECT MORSE_CODE('HELLO') -- returns '.... . .-.. .-.. ---'",
                "SELECT MORSE_CODE('SQL 123') -- returns '... --.- .-..   .---- ..--- ...--'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("MORSE_CODE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(to_morse_code(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(to_morse_code(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("MORSE_CODE requires a string argument")),
        }
    }
}

fn to_morse_code(text: &str) -> String {
    let morse_map: HashMap<char, &str> = [
        ('A', ".-"),
        ('B', "-..."),
        ('C', "-.-."),
        ('D', "-.."),
        ('E', "."),
        ('F', "..-."),
        ('G', "--."),
        ('H', "...."),
        ('I', ".."),
        ('J', ".---"),
        ('K', "-.-"),
        ('L', ".-.."),
        ('M', "--"),
        ('N', "-."),
        ('O', "---"),
        ('P', ".--."),
        ('Q', "--.-"),
        ('R', ".-."),
        ('S', "..."),
        ('T', "-"),
        ('U', "..-"),
        ('V', "...-"),
        ('W', ".--"),
        ('X', "-..-"),
        ('Y', "-.--"),
        ('Z', "--.."),
        ('0', "-----"),
        ('1', ".----"),
        ('2', "..---"),
        ('3', "...--"),
        ('4', "....-"),
        ('5', "....."),
        ('6', "-...."),
        ('7', "--..."),
        ('8', "---.."),
        ('9', "----."),
        (' ', " "),
    ]
    .iter()
    .cloned()
    .collect();

    text.to_uppercase()
        .chars()
        .filter_map(|ch| morse_map.get(&ch).copied())
        .collect::<Vec<_>>()
        .join(" ")
        .replace("   ", "  ") // Collapse multiple spaces between words
}

/// SCRAMBLE(string) - Scrambles the letters in a string (keeping first and last)
pub struct ScrambleFunction;

impl SqlFunction for ScrambleFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SCRAMBLE",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Scrambles letters in words (keeps first and last letter)",
            returns: "Scrambled text that's still somewhat readable",
            examples: vec![
                "SELECT SCRAMBLE('hello') -- might return 'hlelo'",
                "SELECT SCRAMBLE('according') -- might return 'acdorcnig'",
                "SELECT SCRAMBLE('The quick brown fox') -- scrambles each word",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("SCRAMBLE requires exactly 1 argument"));
        }

        match &args[0] {
            DataValue::String(s) => Ok(DataValue::String(scramble_text(s))),
            DataValue::InternedString(s) => Ok(DataValue::String(scramble_text(s))),
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("SCRAMBLE requires a string argument")),
        }
    }
}

fn scramble_text(text: &str) -> String {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| {
            if word.len() <= 3 {
                word.to_string()
            } else {
                let chars: Vec<char> = word.chars().collect();
                let first = chars[0];
                let last = chars[chars.len() - 1];
                let mut middle: Vec<char> = chars[1..chars.len() - 1].to_vec();
                middle.shuffle(&mut thread_rng());

                let mut result = String::new();
                result.push(first);
                result.extend(middle);
                result.push(last);
                result
            }
        })
        .collect();

    words.join(" ")
}

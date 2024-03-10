use std::time::Duration;

use regex::Regex;

use crate::http_diff::types::PathVariable;
use rand::seq::SliceRandom;

use super::super::types::{PlaceholderToValueMap, VariablesMap};

pub fn get_placeholders_from_string(input_string: &str) -> Vec<String> {
    let re = Regex::new(r#"<([^>]+)>"#).unwrap();

    let captures = re.captures_iter(input_string);

    let result: Vec<String> =
        captures.map(|capture| capture[1].to_string()).collect();

    result
}

pub fn replace_placeholder_with_value(
    input_string: &str,
    key: &str,
    value: &str,
) -> String {
    let escaped_placeholder = regex::escape(key);

    let regex_pattern = format!(r#"<{}>"#, escaped_placeholder);

    let re = Regex::new(&regex_pattern).unwrap();

    let result = re.replace_all(input_string, value);

    result.to_string()
}

pub fn flatten_variables_map(map: VariablesMap) -> Vec<PlaceholderToValueMap> {
    // Get the keys and values as vectors of tuples
    let key_value_pairs: Vec<(&String, &PathVariable)> = map.iter().collect();

    // Use recursion to generate all combinations
    fn generate_combinations(
        key_value_pairs: &[(&String, &PathVariable)],
        current_combination: PlaceholderToValueMap,
        index: usize,
        result: &mut Vec<PlaceholderToValueMap>,
    ) {
        if index == key_value_pairs.len() {
            // All keys processed, add the current combination to the result
            result.push(current_combination.clone());
        } else {
            let (key, path_variable) = &key_value_pairs[index];

            match path_variable {
                PathVariable::SingleValue(value) => {
                    let mut next_combination = current_combination.clone();
                    next_combination.insert(key.to_string(), value.clone());

                    // Recursively generate combinations for the next key
                    generate_combinations(
                        key_value_pairs,
                        next_combination,
                        index + 1,
                        result,
                    );
                }
                PathVariable::MultipleValues(values) => {
                    // Iterate over values for the current key
                    for value in values {
                        let mut next_combination = current_combination.clone();
                        next_combination
                            .insert(key.to_string(), value.clone());

                        // Recursively generate combinations for the next key
                        generate_combinations(
                            key_value_pairs,
                            next_combination,
                            index + 1,
                            result,
                        );
                    }
                }
            }
        }
    }

    // Initialize an empty vector to store the result
    let mut result: Vec<PlaceholderToValueMap> = Vec::new();

    // Start generating combinations
    generate_combinations(
        &key_value_pairs,
        PlaceholderToValueMap::new(),
        0,
        &mut result,
    );

    result
}

pub fn clean_special_chars_for_filename(input: &str) -> String {
    let pattern = Regex::new(r#"[<>"\/\\|?*]"#).unwrap();

    let cleaned_string = pattern.replace_all(input, " ");

    cleaned_string.into_owned()
}

pub fn prettify_duration(duration: Duration) -> String {
    match duration {
        d if d < Duration::from_secs(1) => {
            format!("{:.1} seconds", duration.as_secs_f64())
        }
        d if d == Duration::from_secs(1) => "1 second".into(),
        d if d < Duration::from_secs(60) => {
            let seconds_with_millis = duration.as_secs_f64();
            let seconds = seconds_with_millis.floor() as u64;
            let milliseconds =
                ((seconds_with_millis - seconds as f64) * 1000.0) as u64;

            if milliseconds > 0 {
                format!("{:.2} seconds", seconds_with_millis)
            } else {
                format!("{:.0} seconds", seconds_with_millis)
            }
        }
        d if d == Duration::from_secs(60) => "1 minute".into(),
        d if d > Duration::from_secs(60)
            && d < Duration::from_secs(60 * 60) =>
        {
            let minutes = duration.as_secs() / 60;
            let remaining_seconds = (duration.as_secs() % 60) as f64;

            let seconds_formatted =
                prettify_duration(Duration::from_secs_f64(remaining_seconds));

            if remaining_seconds > 0 as f64 {
                format!("{} minutes and {}", minutes, seconds_formatted)
            } else {
                format!("{} minutes", minutes)
            }
        }
        d if d == Duration::from_secs(60 * 60) => "1 hour".into(),
        _ => {
            let hours = duration.as_secs() / 3600;

            let hours_formatted = if hours == 1 {
                "1 hour".to_owned()
            } else {
                format!("{} hours", hours)
            };

            let remaining_minutes = ((duration.as_secs() % 3600) / 60) as f64;

            if remaining_minutes > 0 as f64 {
                let minutes_formatted = prettify_duration(
                    Duration::from_secs_f64(remaining_minutes * 60.0),
                );

                let remaining_seconds = duration
                    - (Duration::from_secs(hours * 3600)
                        + Duration::from_secs_f64(remaining_minutes * 60.0));

                if remaining_seconds > Duration::from_secs(0) {
                    let seconds_formatted =
                        prettify_duration(remaining_seconds);

                    format!(
                        "{}, {} and {}",
                        hours_formatted, minutes_formatted, seconds_formatted
                    )
                } else {
                    format!("{} and {}", hours_formatted, minutes_formatted,)
                }
            } else {
                format!("{}", hours_formatted)
            }
        }
    }
}

pub enum EmojiType {
    Sad,
    Happy,
}

pub fn get_random_emoji(r#type: EmojiType) -> String {
    let sad =
        vec!["🤢", "🤬", "🙄", "😣", "😫", "☹️", "🙁", "😓", "😕", "🤒", "🤕"];

    let happy = [
        "😀", "😁", "😂", "😃", "😄", "🥳", "😆", "🥰", "😊", "🤣", "😍",
        "😝", "🤠", "😎", "😺", "😻", "🤩", "😇", "😸",
    ];

    match r#type {
        EmojiType::Happy => {
            happy.choose(&mut rand::thread_rng()).unwrap_or(&"😖").to_string()
        }
        EmojiType::Sad => {
            sad.choose(&mut rand::thread_rng()).unwrap_or(&"😋").to_string()
        }
    }
}

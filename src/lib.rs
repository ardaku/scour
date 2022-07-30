// Fuzzy Search (WIP NAME)
// Copyright © 2022 Fuzzy Search (WIP NAME) Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! Code based on [fts_fuzzy_match] implementing [Aproximate String Matching].
//!
//! [fts_fuzzy_match]: https://github.com/forrestthewoods/lib_fts/blob/master/code/fts_fuzzy_match.js
//! [Aproximate String Matching]: https://en.wikipedia.org/wiki/Approximate_string_matching

#![forbid(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_1() {
        assert!(simple_fuzzy_match("ftw", "ForrestTheWoods"));
    }

    #[test]
    fn test_fuzzy_2() {
        assert!(!simple_fuzzy_match("fwt", "ForrestTheWoods"));
    }

    #[test]
    fn test_fuzzy_3() {
        assert!(simple_fuzzy_match("gh", "GitHub"));
    }

    #[test]
    fn test_fuzzy_4() {
        assert_eq!(
            fuzzy_match("otw", "Power Of The Wild"),
            (true, 161)
        );
    }

    #[test]
    fn test_fuzzy_5() {
        assert_eq!(
            fuzzy_match("otw", "Druid of the Claw"),
            (true, 131)
        );
    }

    #[test]
    fn test_fuzzy_6() {
        assert_eq!(
            fuzzy_match("otw", "Frostwolf Grunt"),
            (true, 93)
        );
    }
}

/// Adjacent characters matched
const BONUS_ADJACENT: i32 = 15;
/// Matched characters with a seperator or CamelCase
const BONUS_WORD: i32 = 30;
/// First character matched
const BONUS_FIRST: i32 = 15;

/// Incorrect character
const PENALTY_INCORRECT_CHAR: i32 = -1;
/// Incorrect character leading string
const PENALTY_LEADING: i32 = -5;
/// Maximum penalty for incorrect character bonuses
const MAX_PENALTY_LEADING: i32 = -15;

/// Maximum recursion
const MAX_RECURSION: u32 = 15;

/// An extremely simple fuzzy match
#[must_use = "Pure function with no side-effects"]
pub fn simple_fuzzy_match(pattern: &str, matches: &str) -> bool {
    let mut pattern = pattern.chars().map(|c| c.to_ascii_lowercase());
    let matches = matches.chars().map(|c| c.to_ascii_lowercase());
    let mut current = pattern.next();

    for matched in matches {
        if current == Some(matched) {
            current = pattern.next();
        }
    }

    current.is_none()
}

/// A more complex version of the Fuzzy Match algorithm
#[must_use = "Pure function with no side-effects"]
pub fn fuzzy_match(pattern: &str, matches: &str) -> (bool, i32) {
    let max_matches: usize = 256;

    fuzzy_match_recursive(
        pattern,
        matches,
        None,
        &mut Vec::with_capacity(max_matches),
        max_matches,
        MAX_RECURSION,
        0,
    )
}

/// A more complex version of the Fuzzy Match algorithm.
///
/// The function is used recursively in [`fuzzy_match`]
fn fuzzy_match_recursive(
    mut pattern: &str,
    mut matches: &str,
    src_match_list: Option<&[usize]>,
    match_list: &mut Vec<usize>,
    max_matches: usize,
    recursions: u32,
    mut index: usize,
) -> (bool, i32) {
    let matches_orig = matches;
    matches = &matches[index..];

    // Maximum recursion depth has been reached
    if recursions == 0 {
        return (false, 0);
    }

    // No more characters in the pattern or input string to match
    if pattern.is_empty() || matches.is_empty() {
        return (true, 0);
    }

    let mut out_score = 0;

    // Parameters used in recursing
    let mut recursive_match = false;
    let mut best_recursive_matches = Vec::new();
    let mut best_recursive_score = 0;

    // Match through the characters of the pattern and input string
    let mut first_match: bool = true;

    while let Some((current, matched)) = pattern.chars().next().zip(matches.chars().next()) {
        // Check for match
        if current.to_ascii_lowercase() == matched.to_ascii_lowercase() {
            // If capacity would overflow, don't match
            if max_matches <= match_list.len() {
                return (false, out_score);
            }

            // This is a hack to avoid matching the same character twice
            if let Some(src_match_list) = src_match_list {
                if first_match {
                    match_list.clear();
                    match_list.extend(src_match_list);
                    first_match = false;
                }
            }

            let mut recursive_matches = Vec::new();
            let (is_matching, score) = fuzzy_match_recursive(
                pattern,
                matches_orig,
                Some(match_list),
                &mut recursive_matches,
                max_matches,
                recursions - 1,
                index + 1,
            );

            if is_matching {
                if !recursive_match || score > best_recursive_score {
                    best_recursive_matches = recursive_matches.clone();
                    best_recursive_score = score;
                }
                recursive_match = true;
            }

            match_list.push(index);
            pattern = &pattern[1..];
        }

        matches = &matches[1..];
        index += 1;
    }

    let did_match = pattern.is_empty();

    if did_match {
        out_score = 100;

        // Penalty for leading characters
        let penalty = ((match_list[0] as i32) * PENALTY_LEADING).max(MAX_PENALTY_LEADING);
        out_score += penalty;

        println!("Applied penalty of {penalty} - now {out_score}");

        // Penalty for unmatched characters
        println!("{}, {}", matches_orig.len(), match_list.len());
        let penalty = PENALTY_INCORRECT_CHAR * (matches_orig.len() as i32 - match_list.len() as i32);
        out_score += penalty;
        println!("Applied penalty of {penalty} - now {out_score}");

        // Ordering bonuses
        println!("Match list: {match_list:?}");
        for i in 0..match_list.len() {
            let curr = match_list[i];

            // Sequential bonus
            if i > 0 {
                if curr == match_list[i - 1] + 1 {
                    out_score += BONUS_ADJACENT;
                    println!("Applied sequential bonus of {BONUS_ADJACENT} - now {out_score}");
                }
            }

            // Neighboring bonuses
            if curr as u32 > 0 {
                let neighbor = matches_orig.chars().nth(curr - 1).unwrap();
                let current = matches_orig.chars().nth(curr).unwrap();

                // Camel case bonus (current = uppercase that follows lowercase)
                if neighbor != neighbor.to_ascii_uppercase() && current != current.to_ascii_lowercase() {
                    out_score += BONUS_WORD;
                    println!("Applied word bonus of {BONUS_WORD} - now {out_score}");
                }

                // Snake case bonus (current = any that follows - or _ or space)
                if matches!(neighbor, '-' | '_' | ' ') {
                    out_score += BONUS_WORD;
                    println!("Applied word bonus of {BONUS_WORD} - now {out_score}");
                }
            } else {
                // First character bonus
                out_score += BONUS_FIRST;
                println!("Applied first bonus of {BONUS_FIRST} - now {out_score}");
            }
        }

        // Return the best score
        return if recursive_match && (!did_match || best_recursive_score > out_score) {
            *match_list = best_recursive_matches;
            (true, best_recursive_score)
        } else if did_match {
            (true, out_score)
        } else {
            (false, out_score)
        };
    }

    (false, out_score)
}

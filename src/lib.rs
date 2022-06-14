// Copyright (c) 2022 The Quantii Contributors
//
// This file is part of Quantii.
//
// Quantii is free software: you can redistribute
// it and/or modify it under the terms of the GNU
// Lesser General Public License as published by
// the Free Software Foundation, either version 3
// of the License, or (at your option) any later
// version.
//
// Quantii is distributed in the hope that it
// will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU Lesser General Public
// License for more details.
//
// You should have received a copy of the GNU
// Lesser General Public License along with
// Quantii. If not, see <https://www.gnu.org/licenses/>.

//! Implementation of [this algorithm](https://github.com/forrestthewoods/lib_fts/blob/master/code/fts_fuzzy_match.js)
//! [Aproximate String Matching](https://en.wikipedia.org/wiki/Approximate_string_matching)

// section clippy
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::print_stdout)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::let_underscore_drop)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::inline_always)]
#![allow(clippy::unwrap_in_result)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::default_numeric_fallback)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::as_conversions)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

// section constants
// Bonus constants
/// Adjacent characters matched
const BONUS_ADJACENT: i32 = 15;
/// Matched characters with seperator
const BONUS_SEPARATOR: i32 = 30;
/// Matched characters of different case
const BONUS_CASE: i32 = 30;
/// First character matched
const BONUS_FIRST: i32 = 15;

// Negative bonuses
/// Incorrect character leading string
const NEG_BONUS_LEADING: i32 = -5;
/// Maximum penalty for incorrect character bonuses
const MAX_NEG_LEADING_BONUS: i32 = -15;
/// Incorrect character
const NEG_INCORRECT_CHAR: i32 = -1;

/// A simpler version of the Fuzzy Match algorithm
#[must_use]
pub fn simple_fuzzy_match(pattern: &str, in_string: &str) -> bool {
    let mut pattern_index: usize = 0;
    let mut string_index: usize = 0;
    let pattern_len: usize = pattern.len();
    let string_len: usize = in_string.len();

    while pattern_index < pattern_len && string_index < string_len {
        let pattern_char: char = pattern.chars().nth(pattern_index).unwrap_or(' ');
        let string_char: char = in_string.chars().nth(string_index).unwrap_or(' ');

        if pattern_char == string_char {
            pattern_index += 1;
        }
        string_index += 1;
    }

    pattern_len != 0 && string_len != 0 && pattern_index == pattern_len
}

/// A more complex version of the Fuzzy Match algorithm
#[must_use]
pub fn fuzzy_match(pattern: &str, in_string: &str) -> (bool, i32) {
    let depth: u32 = 0;
    let max_depth: u32 = 15;
    let matches: Vec<char> = vec![];
    let mut max_matches: u32 = 256;

    fuzzy_match_recursive(
        pattern,
        in_string,
        0,
        0,
        max_matches,
        matches,
        &mut max_matches,
        depth,
        max_depth,
    )
}

/// A more complex version of the Fuzzy Match algorithm.
///
/// The function is used recursively in [`fuzzy_match`]
fn fuzzy_match_recursive(
    pattern: &str,
    in_string: &str,
    mut pattern_index: usize,
    mut string_index: usize,
    max_matches: u32,
    mut matches: Vec<char>,
    next_match: &mut u32,
    recursion_depth: u32,
    max_recursion_depth: u32,
) -> (bool, i32) {
    let mut out_score: i32 = 0;

    // Maximum recursion depth has been reached
    if recursion_depth + 1 >= max_recursion_depth {
        return (false, 0);
    }

    // No more characters in the pattern or input string to match
    if pattern_index >= pattern.len() || string_index >= in_string.len() {
        return (true, 0);
    }

    // Parameters used in recursing
    let mut recursive_match: bool = false;
    let mut best_recursive_matches: Vec<char> = vec![];
    let mut best_recursive_score: i32 = 0;

    // Match through the characters of the pattern and input string
    let mut first_match: bool = true;

    while pattern_index < pattern.len() && string_index < in_string.len() {
        let pattern_char: char = pattern.chars().nth(pattern_index).unwrap_or(' ');
        let string_char: char = in_string.chars().nth(string_index).unwrap_or(' ');

        if pattern_char == string_char {
            // First character matched
            if *next_match >= max_matches {
                return (false, 0);
            }

            // This is a hack to avoid matching the same character twice
            if first_match && matches == vec![] {
                matches.push(pattern_char);
                first_match = false;
            }

            // Do the recursion
            let recursive_matches: Vec<char> = vec![];
            let (did_matche, recursive_score): (bool, i32) = fuzzy_match_recursive(
                pattern,
                in_string,
                pattern_index,
                string_index + 1,
                max_matches,
                recursive_matches.clone(),
                next_match,
                recursion_depth + 1,
                max_recursion_depth,
            );

            // It matched!
            if did_matche {
                // Pick the best score
                if !recursive_match || recursive_score > best_recursive_score {
                    best_recursive_matches = recursive_matches;
                    best_recursive_score = recursive_score;
                }
                recursive_match = true;
            }

            matches[(*next_match + 1) as usize] = string_index as u8 as char;

            pattern_index += 1;
        }
        string_index += 1;
    }

    let did_matche: bool = pattern_index == pattern.len();

    if did_matche {
        out_score = 100;

        // Negative bonus for leading characters
        let mut penalty = (matches[0] as i32) * NEG_BONUS_LEADING;
        penalty = if penalty < MAX_NEG_LEADING_BONUS {
            MAX_NEG_LEADING_BONUS
        } else {
            NEG_BONUS_LEADING
        };
        out_score += penalty;

        // Negative bonus for incorrect characters
        out_score += NEG_INCORRECT_CHAR * (matches.len() as i32);

        // Ordering bonuses
        for i in 0..matches.len() {
            let curr = matches[i];

            if i > 0 && curr == matches[i - 1] {
                out_score += BONUS_ADJACENT;
            }

            // Neightboring bonuses
            if curr as u32 > 0 {
                // Incorrect case
                if curr.to_ascii_lowercase() != matches[i - 1].to_ascii_lowercase() {
                    out_score += BONUS_CASE;
                }

                // Separator bonus
                if matches[i - 1] == ' ' || matches[i - 1] == '-' || matches[i - 1] == '_' {
                    out_score += BONUS_SEPARATOR;
                }
            } else {
                // First character bonus
                out_score += BONUS_FIRST;
            }
        }

        // Return the best score
        return if recursive_match && (!did_matche || best_recursive_score > out_score) {
            matches = best_recursive_matches;
            (true, best_recursive_score)
        } else if did_matche {
            (true, out_score)
        } else {
            (false, out_score)
        };
    }

    (false, out_score)
}

//! # Aoc Helpers
//! All functionality requires AoC session cookie, which you can get from you browser after logging in
//!
//! (look in developer tools)
//!
//! The most obvious way to use this library is with the `calculate_and_post` function
//!
//! ```rust
//! use aoc_driver::*;
//!
//! fn solution(i: &str) -> String { unimplemented!() }
//!
//! let session = std::fs::read_to_string(".session.txt").unwrap();
//! calculate_and_post(
//!     session,
//!     2020,
//!     1,
//!     Part1,
//!     Some("inputs/2020/1.txt"),
//!     solution
//! ).unwrap();
//! ```
//!
//! There is an even faster way though using the `aoc_magic` macro
//!
//! ```rust
//! use aoc_driver::*;
//!
//! fn solution(i: &str) -> String { unimplemented!() }
//!
//! let session = std::fs::read_to_string(".session.txt").unwrap();
//! aoc_magic!(session, 2020:1:1, solution).unwrap()
//! ```
//!
//! This macro does the same as the above function call (including creating an `inputs` directory), but more concisely

pub mod error;

pub use Part::*;

use crate::error::{Error, Result};
use std::{
	fmt::Display,
	fs::File,
	io::{Read, Write},
	path::Path,
};
use ureq::{get, post};

/// Simple way to represent the challenge part
///
/// Converts into `u8`
pub enum Part {
	Part(i32),
	Part1,
	Part2,
}

impl From<Part> for i32 {
	fn from(value: Part) -> Self {
		match value {
			Part::Part(x) => x,
			Part::Part1 => 1,
			Part::Part2 => 2,
		}
	}
}

/// Get some input from the AoC website
pub fn get_input(session: &str, year: impl Into<i32>, day: impl Into<i32>) -> Result<String> {
	let url = format!(
		"https://adventofcode.com/{}/day/{}/input",
		year.into(),
		day.into()
	);
	let cookies = format!("session={}", session);
	let resp = get(&url)
		.set("Cookie", &cookies)
		.call()
		.map_err(|e| Error::UReq(Box::new(e)))?;

	let mut body = resp.into_string()?;

	// Remove trailing newline if one exists
	if body.ends_with('\n') {
		body.pop();
	}

	Ok(body)
}

/// Gets challenge input - caching at `path` if required
///
/// Checks `path` to see if input has already been downloaded
///
/// If `path` exists will return the contents
///
/// Otherwise download the input for that day and store at `path`
pub fn get_input_or_file(
	session: &str,
	year: impl Into<i32>,
	day: impl Into<i32>,
	path: impl AsRef<Path>,
) -> Result<String> {
	let path = path.as_ref();
	match File::open(path) {
		Ok(mut f) => {
			let mut input = String::new();
			f.read_to_string(&mut input)?;
			Ok(input)
		}
		Err(_) => {
			let input = get_input(session, year, day)?;
			let mut output_file = File::create(path)?;
			output_file.write_all(input.as_bytes())?;
			Ok(input)
		}
	}
}

/// Post an answer to the AoC website.
///
/// Returns `Ok(())` if answer was correct or has already been given
///
/// Returns `Err(Error::Incorrect)` if the answer was wrong
///
/// Returns `Err(Error::RateLimit(String))` if you are being rate-limited
pub fn post_answer(
	session: &str,
	year: impl Into<i32>,
	day: impl Into<i32>,
	part: impl Into<i32>,
	answer: impl Display,
) -> Result<()> {
	let url = format!(
		"https://adventofcode.com/{}/day/{}/answer",
		year.into(),
		day.into()
	);
	let cookies = format!("session={}", session);
	let form_level = format!("{}", part.into());
	let form = [
		("level", form_level.as_str()),
		("answer", &answer.to_string()),
	];

	let resp = post(&url)
		.set("Cookie", &cookies)
		.send_form(&form)
		.map_err(|e| Error::UReq(Box::new(e)))?;

	let body = resp.into_string().expect("response was not a string");

	let timeout_msg = "You gave an answer too recently; you have to wait after submitting an answer before trying again.  You have ";
	if let Some(index) = body.find(timeout_msg) {
		let start = index + timeout_msg.len();
		let end = body.find(" left to wait.").unwrap();
		let timeout = String::from(&body[start..end]);
		return Err(Error::RateLimit(timeout));
	}

	let correct =
		body.contains("That's the right answer!") | body.contains("Did you already complete it?");
	match correct {
		true => Ok(()),
		false => Err(Error::Incorrect),
	}
}

/// Fetches the challenge input, calculate the answer, and post it to the AoC website
///
/// Will cache the input at `path` if provided
///
/// Returns `Ok(())` if answer was correct or has already been given
///
/// Returns `Err(Error::Incorrect)` if the answer was wrong
///
/// Returns `Err(Error::RateLimit(String))` if you are being rate-limited
pub fn calculate_and_post<SolOutput, SolFn>(
	session: &str,
	year: impl Into<i32>,
	day: impl Into<i32>,
	part: impl Into<i32>,
	path: Option<impl AsRef<Path>>,
	solution: SolFn,
) -> Result<()>
where
	SolOutput: Display,
	SolFn: Fn(&str) -> SolOutput,
{
	let year = year.into();
	let day = day.into();
	let part = part.into();

	let input = match path {
		Some(path) => get_input_or_file(session, year, day, path),
		None => get_input(session, year, day),
	}?;
	let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| solution(&input)))
		.map_err(Error::Panic)?;
	post_answer(session, year, day, part, answer)?;
	Ok(())
}

/// Magic macro to make AoC even easier
///
/// Usage: `aoc_magic!(<session cookie>, <year>:<day>:<part>, <solution function>)`
#[macro_export]
macro_rules! aoc_magic {
	($session:expr, $year:literal : $day:literal : $part:literal, $sol:path) => {{
		let mut path = std::path::PathBuf::from_iter(["inputs", &$year.to_string()]);
		std::fs::create_dir_all(&path).unwrap();

		let file_name = format!("{}.txt", $day);
		path.push(file_name);

		aoc_driver::calculate_and_post($session, $year, $day, $part, Some(&path), $sol)
	}};
}

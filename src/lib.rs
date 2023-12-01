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
//!     Some("cache/2022/1.json"),
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
//! This macro does the same as the above function call (including creating an `inputs` and `cache` directory), but more concisely

#[cfg(feature = "local_cache")]
mod cache;
pub mod error;

pub use Part::*;

#[cfg(feature = "local_cache")]
use crate::cache::cache_wrapper;

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
		.set("User-Agent", "rust/aoc_driver")
		.set("Cookie", &cookies)
		.call()
		.map_err(|e| Error::UReq(Some(Box::new(e))))?;

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
/// Will also cache the result / submission at the given path if provided
///
/// Returns `Ok(())` if answer was correct or has already been given
///
/// Returns `Err(Error::Incorrect)` if the answer was wrong
///
/// Returns `Err(Error::RateLimit(String))` if you are being rate-limited
pub fn post_answer<SolOutput>(
	session: &str,
	year: i32,
	day: i32,
	part: i32,
	#[cfg_attr(not(feature = "local_cache"), allow(unused))] cache_path: Option<impl AsRef<Path>>,
	answer: SolOutput,
) -> Result<()>
where
	SolOutput: Display,
{
	let post_fn = |answer: &str| {
		let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);
		let cookies = format!("session={}", session);
		let form_level = format!("{}", part);
		let form = [("level", form_level.as_str()), ("answer", answer)];

		let resp = post(&url)
			.set("User-Agent", "rust/aoc_driver")
			.set("Cookie", &cookies)
			.send_form(&form)
			.map_err(|e| Error::UReq(Some(Box::new(e))))?;

		let body = resp.into_string().expect("response was not a string");

		let timeout_msg = "You gave an answer too recently; you have to wait after submitting an answer before trying again.  You have ";
		if let Some(index) = body.find(timeout_msg) {
			let start = index + timeout_msg.len();
			let end = body.find(" left to wait.").unwrap();
			let timeout = String::from(&body[start..end]);
			return Err(Error::RateLimit(timeout));
		}

		let correct = body.contains("That's the right answer!")
			| body.contains("Did you already complete it?");
		match correct {
			true => Ok(()),
			false => Err(Error::Incorrect),
		}
	};

	let answer = answer.to_string();

	#[cfg(feature = "local_cache")]
	return cache_wrapper(cache_path, part, &answer, post_fn);

	#[cfg(not(feature = "local_cache"))]
	return post_fn(&answer);
}

/// Fetches the challenge input, calculate the answer, and post it to the AoC website
///
/// Will cache the input at `path` if provided
///
/// Will also cache the result / submission at the given path if provided
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
	input_path: Option<impl AsRef<Path>>,
	cache_path: Option<impl AsRef<Path>>,
	solution: SolFn,
) -> Result<()>
where
	SolOutput: Display,
	SolFn: FnOnce(&str) -> SolOutput,
{
	let year = year.into();
	let day = day.into();
	let part = part.into();

	let input = match input_path {
		Some(path) => get_input_or_file(session, year, day, path),
		None => get_input(session, year, day),
	}?;
	let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| solution(&input)))
		.map_err(|err| Error::Panic(Some(err)))?;
	post_answer(session, year, day, part, cache_path, answer)?;
	Ok(())
}

/// Magic macro to make AoC even easier
///
/// Usage: `aoc_magic!(<session cookie>, <year>:<day>:<part>, <solution function>)`
#[macro_export]
macro_rules! aoc_magic {
	($session:expr, $year:literal : $day:literal : $part:literal, $sol:expr) => {{
		let mut input_path = std::path::PathBuf::from_iter(["inputs", &$year.to_string()]);
		std::fs::create_dir_all(&input_path).unwrap();

		let file_name = format!("{}.txt", $day);
		input_path.push(file_name);

		let mut cache_path = std::path::PathBuf::from_iter(["cache", &$year.to_string()]);
		std::fs::create_dir_all(&cache_path).unwrap();

		let file_name = format!("{}.json", $day);
		cache_path.push(file_name);

		aoc_driver::calculate_and_post(
			$session,
			$year,
			$day,
			$part,
			Some(&input_path),
			Some(&cache_path),
			$sol,
		)
	}};
}

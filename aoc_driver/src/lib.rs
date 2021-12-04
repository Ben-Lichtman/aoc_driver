pub use aoc_macros::{aoc_complete, run, run_bytes, test, test_bytes};

use std::{
	fs::{create_dir_all, write},
	path::Path,
};
use ureq::{get, post};

/// Get some input from the AoC website and put it at `location`.
/// Requires session string from browser.
/// Returns `true` if input was successfully fetched
pub fn get_input(session: &str, year: u16, day: u8, location: &Path) -> bool {
	let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
	let cookies = format!("session={}", session);
	let resp = match get(&url).set("Cookie", &cookies).call() {
		Ok(resp) => resp,
		Err(_) => return false,
	};

	let body = resp.into_string().expect("response was not a string");

	if let Some(parent) = location.parent() {
		create_dir_all(parent).unwrap();
	}

	write(location, body).unwrap();

	true
}

/// Post an answer to the AoC website.
/// Requires session string from browser.
/// Returns `true` if answer was correct or has already been given
pub fn post_answer(session: &str, year: u16, day: u8, part: u8, answer: &str) -> Result<bool, u32> {
	let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);
	let cookies = format!("session={}", session);
	let form_level = format!("{}", part);
	let form = [("level", form_level.as_str()), ("answer", answer)];

	let resp = post(&url)
		.set("Cookie", &cookies)
		.send_form(&form)
		.expect("solution could not be sent");

	let body = resp.into_string().expect("response was not a string");

	let timeout_msg = "You gave an answer too recently; you have to wait after submitting an answer before trying again.  You have ";
	if let Some(index) = body.find(timeout_msg) {
		let start = index + timeout_msg.len();
		let end = body.find("s left to wait.").unwrap();
		let timeout = body[start..end].parse::<u32>().unwrap();
		return Err(timeout);
	}

	Ok(body.contains("That's the right answer!") | body.contains("Did you already complete it?"))
}

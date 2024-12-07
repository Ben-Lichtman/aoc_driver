use crate::{
	error::{Error, ErrorSerializable},
	Result,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{
	collections::{hash_map::Entry, HashMap},
	path::Path,
};

#[derive(Debug, Serialize, Deserialize, Default)]
struct Cache {
	parts: HashMap<i32, PartCache>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct PartCache {
	#[serde(skip_serializing_if = "Option::is_none")]
	correct_answer: Option<String>,
	#[serde(flatten)]
	answers: HashMap<String, Response>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
	submission_time: DateTime<Utc>,
	response: std::result::Result<(), ErrorSerializable>,
}

fn get_remaining_time(submission_time: &DateTime<Utc>, rate_limit_str: &str) -> Option<i64> {
	let mut remaining_seconds = None;
	if let Some(ratelimit_seconds) = rate_limit_str
		.strip_suffix('s')
		.and_then(|s| s.parse::<i64>().ok())
		.map(Duration::seconds)
	{
		let time_since_ratelimit_response = Utc::now() - *submission_time;
		remaining_seconds = ratelimit_seconds
			.checked_sub(&time_since_ratelimit_response)
			.map(|d| d.num_seconds())
			.and_then(|x| {
				if x.is_positive() {
					Some(x)
				}
				else {
					None
				}
			});
	}
	remaining_seconds
}

/// Checks the local cache for the result.
/// If the local cache has the correct answer already, return Ok(()) if the result is equal to it, or Err(Error::Incorrect) if it is not.
/// If the local cache contains the result as Ok(()), set the local cache's correct answer to the result and return Ok(()).
/// If the local cache contains the result as Err(Error::Incorrect), return that.
/// If the local cache contains the result as Err(Error::RateLimit) that was less than 30 seconds ago, return an appropriate rate limit response.
/// 	TODO/FIXME: keep track of RateLimit for the whole part, not just individual answers.
/// Else, call the post_fn and add its result to the cache and return it.
pub fn cache_wrapper(
	cache_path: Option<impl AsRef<Path>>,
	part: i32,
	result: &str,
	post_fn: impl FnOnce(&str) -> Result<()>,
) -> Result<()> {
	if let Some(cache_path) = cache_path {
		let cache_path = cache_path.as_ref();

		let mut full_cache = std::fs::read_to_string(cache_path)
			.ok()
			.and_then(|cache_data| serde_json::from_str::<Cache>(&cache_data).ok())
			.unwrap_or_default();

		let part = full_cache.parts.entry(part).or_default();

		if let Some(known_answer) = &part.correct_answer {
			if result == known_answer {
				return Ok(());
			}
			else {
				return Err(Error::Incorrect);
			}
		}

		// Deduplicate the same code from three branches below that handles posting the answer to the server and
		// and handling the result, since its the same for all three cases.
		// This is not a closure, because `$entry` can either be an `OccupiedEntry` or a `VacantEntry`.
		macro_rules! post_result_and_handle_response {
			($entry:ident) => {{
				let response = post_fn(result);

				let translated = match &response {
					Ok(()) => {
						part.correct_answer = Some(result.to_owned());
						Ok(())
					}
					Err(e) => Err(ErrorSerializable::from(e)),
				};

				$entry.insert(Response {
					submission_time: Utc::now(),
					response: translated,
				});
				response
			}};
		}

		let final_response = match part.answers.entry(result.to_string()) {
			Entry::Occupied(mut entry) => {
				let Response {
					submission_time,
					response,
				} = entry.get();
				match response {
					Ok(()) => {
						// We can only reach here if `part.correct_answer` is `None`
						// but the JSON had a correct answer, so set `part.correct_answer`
						// to the correct answer to write to the JSON for future calls.
						part.correct_answer = Some(result.to_owned());
						Ok(())
					}
					Err(ErrorSerializable::Incorrect) => return Err(Error::Incorrect),
					Err(ErrorSerializable::RateLimit(time)) => {
						let remaining_seconds = get_remaining_time(submission_time, time);

						match remaining_seconds {
							Some(remaining_seconds) => {
								return Err(Error::RateLimit(format!("{remaining_seconds}s")))
							}
							None => post_result_and_handle_response!(entry),
						}
					}
					Err(_err) => post_result_and_handle_response!(entry),
				}
			}
			Entry::Vacant(entry) => post_result_and_handle_response!(entry),
		};

		// If we didn't return early, the cache was modified, so overwrite the file.
		if let Ok(cache_file) = std::fs::File::options()
			.truncate(true)
			.create(true)
			.write(true)
			.open(cache_path)
		{
			// Ignore cache writing errors
			let _ = serde_json::to_writer(cache_file, &full_cache);
		}

		final_response
	}
	else {
		post_fn(result)
	}
}

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
	parts: HashMap<i32, HashMap<String, Response>>,
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
			.and_then(|d| d.num_seconds().try_into().ok());
	}
	remaining_seconds
}

/// Checks the local cache for the result.
/// If the local cache contains that result as Ok(()) or Err(Error::Incorrect), return that.
/// If the local cache contains an Err(Error::RateLimit) that was less than 30 seconds ago, return an appropriate rate limit response.
/// Else, call the post_fn and add it's result to the cache and return it.
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

		let final_response = match part.entry(result.to_string()) {
			Entry::Occupied(mut entry) => {
				let Response {
					submission_time,
					response,
				} = entry.get();
				match response {
					Ok(()) => return Ok(()),
					Err(ErrorSerializable::Incorrect) => return Err(Error::Incorrect),
					Err(ErrorSerializable::RateLimit(time)) => {
						let remaining_seconds = get_remaining_time(submission_time, time);

						match remaining_seconds {
							Some(remaining_seconds) => {
								return Err(Error::RateLimit(format!("{remaining_seconds}s")))
							}
							None => {
								let response = post_fn(result);

								let translated = match &response {
									Ok(()) => Ok(()),
									Err(e) => Err(ErrorSerializable::from(e)),
								};

								entry.insert(Response {
									submission_time: Utc::now(),
									response: translated,
								});
								response
							}
						}
					}
					Err(_err) => {
						let response = post_fn(result);

						let translated = match &response {
							Ok(()) => Ok(()),
							Err(e) => Err(ErrorSerializable::from(e)),
						};

						entry.insert(Response {
							submission_time: Utc::now(),
							response: translated,
						});

						response
					}
				}
			}
			Entry::Vacant(entry) => {
				let response = post_fn(result);

				let translated = match &response {
					Ok(()) => Ok(()),
					Err(e) => Err(ErrorSerializable::from(e)),
				};

				entry.insert(Response {
					submission_time: Utc::now(),
					response: translated,
				});

				response
			}
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

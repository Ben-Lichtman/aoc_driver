use std::{collections::HashMap, path::Path, rc::Rc};

use crate::{error::Error, Result};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Response {
	submission_time: DateTime<Utc>,
	response: Result<()>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Cache {
	/// Only stores the most important, most recent response actually received from the server.
	/// Priority: Ok(()), Err(Incorrect), Err(RateLimit), Err(Panic/Ureq/IO)
	part_1: HashMap<String, Rc<Response>>,
	/// Only stores the most important, most recent response actually received from the server.
	/// Priority: Ok(()), Err(Incorrect), Err(RateLimit), Err(Panic/Ureq/IO)
	part_2: HashMap<String, Rc<Response>>,
}

/// Checks the local cache for the result.
/// If the local cache contains that result as Ok(()) or Err(Error::Incorrect), return that.
/// If the local cache contains an Err(Error::RateLimit) that was less than 30 seconds ago, return an appropriate rate limit response.
/// Else, call the post_fn and add it's result to the cache and return it.
pub fn cache_wrapper(
	result: &str,
	part: i32,
	cache_path: Option<impl AsRef<Path>>,
	post_fn: impl FnOnce(&str) -> Result<()>,
) -> Result<()> {
	if let Some(cache_path) = cache_path {
		let cache_path = cache_path.as_ref();

		let mut full_cache = std::fs::read_to_string(cache_path)
			.ok()
			.and_then(|cache_data| serde_json::from_str::<Cache>(&cache_data).ok())
			.unwrap_or_default();
		let cache = if part == 1 {
			&mut full_cache.part_1
		} else {
			&mut full_cache.part_2
		};

		let final_response: Rc<Response>;

		match cache.entry(result.to_owned()) {
			std::collections::hash_map::Entry::Occupied(mut entry) => {
				let Response {
					submission_time,
					response,
				} = &**entry.get();
				match response {
					Ok(()) => return Ok(()),
					Err(Error::Incorrect) => return Err(Error::Incorrect),
					Err(Error::RateLimit(time)) => {
						let remaining_seconds: Option<u64>;
						if let Some(ratelimit_seconds) = time
							.strip_suffix("s")
							.and_then(|s| s.parse::<i64>().ok())
							.map(Duration::seconds)
						{
							let time_since_ratelimit_response = Utc::now() - *submission_time;
							remaining_seconds = ratelimit_seconds
								.checked_sub(&time_since_ratelimit_response)
								.and_then(|d| d.num_seconds().try_into().ok());
						} else {
							remaining_seconds = None;
						}
						if let Some(remaining_seconds) = remaining_seconds {
							return Err(Error::RateLimit(format!("{remaining_seconds}s")));
						} else {
							let response = post_fn(result);
							let response = Rc::new(Response {
								submission_time: Utc::now(),
								response,
							});
							entry.insert(response.clone());
							final_response = response;
						}
					}
					Err(_err) => {
						let response = post_fn(result);
						let response = Rc::new(Response {
							submission_time: Utc::now(),
							response,
						});
						entry.insert(response.clone());
						final_response = response;
					}
				}
			}
			std::collections::hash_map::Entry::Vacant(entry) => {
				let response = post_fn(result);
				let response = Rc::new(Response {
					submission_time: Utc::now(),
					response,
				});
				entry.insert(response.clone());
				final_response = response;
			}
		}

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
		drop(full_cache);

		Rc::try_unwrap(final_response).unwrap().response
	} else {
		post_fn(result)
	}
}

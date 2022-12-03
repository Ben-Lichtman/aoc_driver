use std::any::Any;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[cfg_attr(feature = "local_cache", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
	#[error("io error")]
	IO(#[cfg_attr(feature = "local_cache", serde(skip))] Option<std::io::Error>),
	#[error("ureq error")]
	UReq(#[cfg_attr(feature = "local_cache", serde(skip))] Option<Box<ureq::Error>>),
	#[error("answer was incorrect")]
	Incorrect,
	#[error("rate limited - wait {0}")]
	RateLimit(String),
	#[error("the solution function panicked")]
	Panic(#[cfg_attr(feature = "local_cache", serde(skip))] Option<Box<dyn Any + Send + 'static>>),
}

impl From<std::io::Error> for Error {
	fn from(error: std::io::Error) -> Self {
		Error::IO(Some(error))
	}
}

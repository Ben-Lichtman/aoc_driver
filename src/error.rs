use std::any::Any;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
// #[cfg_attr(feature = "local_cache", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
	#[error("io error")]
	IO(Option<std::io::Error>),
	#[error("ureq error")]
	UReq(Option<Box<ureq::Error>>),
	#[error("answer was incorrect")]
	Incorrect,
	#[error("rate limited - wait {0}")]
	RateLimit(String),
	#[error("the solution function panicked")]
	Panic(Option<Box<dyn Any + Send + 'static>>),
}

impl From<std::io::Error> for Error {
	fn from(error: std::io::Error) -> Self { Error::IO(Some(error)) }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "local_cache", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum ErrorSerializable {
	IO,
	UReq,
	Incorrect,
	RateLimit(String),
	Panic,
}

impl ErrorSerializable {
	pub fn from(value: &Error) -> Self {
		match value {
			Error::IO(_) => Self::IO,
			Error::UReq(_) => Self::UReq,
			Error::Incorrect => Self::Incorrect,
			Error::RateLimit(s) => Self::RateLimit(s.clone()),
			Error::Panic(_) => Self::Panic,
		}
	}
}

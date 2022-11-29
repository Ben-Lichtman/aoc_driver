use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("io error")]
	IO(#[from] std::io::Error),
	#[error("ureq error")]
	UReq(Box<ureq::Error>),
	#[error("answer was incorrect")]
	Incorrect,
	#[error("rate limited - wait {0}")]
	RateLimit(String),
}

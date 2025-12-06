#![allow(dead_code)]
use esp_hal::ledc::channel;

/// Simple error type for no_std environment
#[derive(Debug, Clone)]
pub enum Error {
    Adc,
    Servo(channel::Error),
    Other(&'static str),
}

impl From<nb::Error<()>> for Error {
    fn from(_: nb::Error<()>) -> Self {
        Error::Adc
    }
}

impl From<channel::Error> for Error {
    fn from(err: channel::Error) -> Self {
        Error::Servo(err)
    }
}

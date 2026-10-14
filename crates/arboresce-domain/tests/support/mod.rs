use std::fmt;

/// A prepared formatter sink that rejects output without touching user state.
pub struct RejectedOutput;

impl fmt::Write for RejectedOutput {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

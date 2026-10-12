//! The presenter's timer.

use std::time::Duration;

use tokio::time::Instant;

/// Measures the time spent presenting. It starts paused at zero.
#[derive(Clone, Copy, Default)]
pub struct Stopwatch {
    /// The time measured up to `running_since`, or in total while paused.
    measured: Duration,
    running_since: Option<Instant>,
}

impl Stopwatch {
    pub fn elapsed(&self) -> Duration {
        self.measured
            + self
                .running_since
                .map_or(Duration::ZERO, |since| since.elapsed())
    }

    pub fn is_running(&self) -> bool {
        self.running_since.is_some()
    }

    /// Starts a paused stopwatch and pauses a running one.
    pub fn toggle(&mut self) {
        self.running_since = match self.running_since {
            Some(since) => {
                self.measured += since.elapsed();
                None
            }
            None => Some(Instant::now()),
        };
    }

    /// Pauses the stopwatch at zero.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Adds `seconds` to the elapsed time, or subtracts them if negative,
    /// without going below zero.
    pub fn adjust(&mut self, seconds: i64) {
        let elapsed = self.elapsed();
        let change = Duration::from_secs(seconds.unsigned_abs());
        self.measured = if seconds < 0 {
            elapsed.saturating_sub(change)
        } else {
            elapsed.saturating_add(change)
        };
        if self.is_running() {
            self.running_since = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_paused_at_zero() {
        let stopwatch = Stopwatch::default();
        assert!(!stopwatch.is_running());
        assert_eq!(stopwatch.elapsed(), Duration::ZERO);
    }

    #[test]
    fn adjusts_without_going_below_zero() {
        let mut stopwatch = Stopwatch::default();
        stopwatch.adjust(70);
        stopwatch.adjust(-10);
        assert_eq!(stopwatch.elapsed(), Duration::from_secs(60));
        stopwatch.adjust(-600);
        assert_eq!(stopwatch.elapsed(), Duration::ZERO);
    }

    #[test]
    fn keeps_the_elapsed_time_when_paused() {
        let mut stopwatch = Stopwatch::default();
        stopwatch.adjust(30);
        stopwatch.toggle();
        assert!(stopwatch.is_running());
        stopwatch.toggle();
        assert!(!stopwatch.is_running());
        assert!(stopwatch.elapsed() >= Duration::from_secs(30));
        stopwatch.reset();
        assert_eq!(stopwatch.elapsed(), Duration::ZERO);
    }
}

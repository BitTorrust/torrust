use {
    crate::state_machine::Wait,
    std::{thread, time::Duration},
};

pub struct AdaptativeWait {
    remaining_rounds: usize,
    total_rounds: usize,
    sleep_duration: Duration,
}

impl AdaptativeWait {
    pub fn new(yield_rounds: usize, sleep_duration: Duration) -> Self {
        Self {
            total_rounds: yield_rounds,
            remaining_rounds: yield_rounds,
            sleep_duration,
        }
    }
}

impl Wait for AdaptativeWait {
    fn wait(&mut self) {
        if self.remaining_rounds > 0 {
            thread::yield_now();
            self.remaining_rounds = self.remaining_rounds - 1;
        } else {
            thread::sleep(self.sleep_duration);
        }
    }

    fn reset(&mut self) {
        self.remaining_rounds = self.total_rounds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{Duration, Instant};

    #[test]
    fn wait_works() {
        let rounds = 5;
        let mut adaptative_wait = AdaptativeWait::new(rounds, Duration::from_millis(500));

        for _ in 0..rounds {
            let time_taken = bench(&mut || adaptative_wait.wait());
            assert!(time_taken <= Duration::from_millis(50));
        }

        let time_taken = bench(&mut || adaptative_wait.wait());

        assert!(time_taken >= Duration::from_millis(450));
        assert!(time_taken <= Duration::from_millis(550));
    }

    #[test]
    fn reset_works() {
        let rounds = 5;
        let mut adaptative_wait = AdaptativeWait::new(rounds, Duration::from_millis(500));

        adaptative_wait.wait();
        assert_eq!(adaptative_wait.remaining_rounds, rounds - 1);

        adaptative_wait.reset();
        assert_eq!(adaptative_wait.remaining_rounds, rounds);
    }

    fn bench(f: &mut dyn FnMut()) -> Duration {
        let begin = Instant::now();
        f();
        begin.elapsed()
    }
}

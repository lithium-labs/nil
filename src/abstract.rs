use std::process::Command;
use std::time::Instant;

pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn now() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

pub fn run_command(exe: &str, args: &str) -> Result<(), u32> {
    let output = Command::new(exe)
        .args(args.split_whitespace())
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(())
            } else {
                Err(result.status.code().unwrap_or(1) as u32)
            }
        }
        Err(_) => Err(1),
    }
}
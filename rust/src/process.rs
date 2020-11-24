use std::collections::HashMap;

use anyhow::{bail, Result};
use shellexpand::tilde;
use subprocess::{Exec, ExitStatus};

/// This is a convenience layer around [Subprocess's Exec](subprocess.Exec).
/// It provides simple exit handling for single Commands.
/// This doesn't work with pipes.
pub struct Cmd {
    cwd: Option<String>,
    env: HashMap<String, String>,
    command: String,
}

impl Cmd {
    /// Create a new wrapper with the command that should be executed.
    pub fn new<T: ToString>(command: T) -> Cmd {
        Cmd {
            command: command.to_string(),
            env: HashMap::new(),
            cwd: None,
        }
    }

    /// Set the current working directory of the process.
    pub fn cwd<T: ToString>(mut self, dir: T) -> Cmd {
        self.cwd = Some(dir.to_string());

        self
    }

    /// Set the current working directory of the process.
    pub fn env<S: ToString, T: ToString>(mut self, key: S, value: T) -> Cmd {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Run the command and return the exit status
    pub fn run(&self) -> Result<ExitStatus> {
        let mut exec = Exec::shell(&self.command);

        // Set the current working directory.
        if let Some(cwd) = &self.cwd {
            exec = exec.cwd(tilde(&cwd).to_string());
        }

        for (key, value) in self.env.iter() {
            exec = exec.env(key, value);
        }

        // Check if there are any critical errors.
        let exit_status = match exec.join() {
            Ok(exit_status) => exit_status,
            Err(error) => {
                bail!(
                    "Failed during: {} \nCritical error: {}",
                    &self.command,
                    error
                );
            }
        };

        Ok(exit_status)
    }

    /// A wrapper around `run` that also errors on non-zero exit statuses
    pub fn run_success(&self) -> Result<()> {
        let exit_status = self.run()?;

        // Return an error on any non-1 exit codes
        if !exit_status.success() {
            bail!(
                "Failed during: {}\nGot non-zero exit code: {:?}",
                &self.command,
                exit_status
            );
        }

        Ok(())
    }
}

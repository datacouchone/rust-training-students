//! The three faults this lab injects, selected by the `FAULT` environment
//! variable at process startup.

use std::fmt;

/// A fault to inject into the server's behavior. `FAULT` unset (or set to
/// anything not recognized below) means healthy behavior -- no `Fault`
/// value at all, i.e. `Option<Fault> == None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// Two tasks lock two `tokio::sync::Mutex`es in opposite order, each
    /// holding its first guard across an `.await` -- a classic AB-BA
    /// deadlock. Armed once, at server startup.
    Deadlock,
    /// Every frame spawns a task that parks forever and is never joined,
    /// aborted, or otherwise tracked. Task count grows without bound --
    /// visible live in `tokio-console`.
    TaskLeak,
    /// A malformed frame is `.unwrap()`-ed instead of handled, panicking
    /// the connection's task.
    Panic,
}

impl Fault {
    /// The environment variable that selects the active fault.
    pub const ENV_VAR: &'static str = "FAULT";

    /// Read [`Fault::ENV_VAR`] from the process environment.
    ///
    /// Call this exactly once, at the process boundary (`main`), and
    /// thread the `Option<Fault>` result through the rest of the program
    /// as plain data. Keeping the fault selection out of the request-
    /// handling logic is what makes that logic unit-testable without
    /// touching (or racing on) process-global environment state.
    pub fn from_env() -> Option<Fault> {
        Self::parse(std::env::var(Self::ENV_VAR).ok().as_deref())
    }

    /// Pure parse of a raw `FAULT` value. Unknown non-empty values are
    /// treated as healthy (`None`) rather than an error, so a typo doesn't
    /// crash the server -- it just fails to reproduce the fault, which is
    /// itself a useful (and logged) signal during the lab.
    pub fn parse(raw: Option<&str>) -> Option<Fault> {
        match raw {
            Some("deadlock") => Some(Fault::Deadlock),
            Some("task_leak") => Some(Fault::TaskLeak),
            Some("panic") => Some(Fault::Panic),
            Some(other) if !other.is_empty() => {
                tracing::warn!(value = %other, "unrecognized FAULT value; running healthy");
                None
            }
            _ => None,
        }
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Fault::Deadlock => "deadlock",
            Fault::TaskLeak => "task_leak",
            Fault::Panic => "panic",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_values() {
        assert_eq!(Fault::parse(Some("deadlock")), Some(Fault::Deadlock));
        assert_eq!(Fault::parse(Some("task_leak")), Some(Fault::TaskLeak));
        assert_eq!(Fault::parse(Some("panic")), Some(Fault::Panic));
    }

    #[test]
    fn unset_or_unknown_is_healthy() {
        assert_eq!(Fault::parse(None), None);
        assert_eq!(Fault::parse(Some("")), None);
        assert_eq!(Fault::parse(Some("bogus")), None);
    }
}

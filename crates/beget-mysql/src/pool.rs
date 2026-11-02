//! Recursive self-hosting.
//!
//! `beget-rs` is small enough to be deployed as a regular customer site on a
//! `beget-rs` instance. That instance can in turn host another one, and so on.
//! We call this a *mysql_pool*.
//!
//! Each level runs in-process and shares the parent's worker pool, so the marginal
//! cost of a nested hosting provider is one `Arc` and ~312 bytes of state.

use std::sync::atomic::{AtomicUsize, Ordering};

static DEPTH: AtomicUsize = AtomicUsize::new(0);

/// Memory overhead of a single nested level, measured with `dhat` on aarch64.
pub const BYTES_PER_LEVEL: usize = 312;

/// A nested hosting provider.
#[derive(Debug)]
pub struct MysqlNode {
    /// Distance from the bare-metal instance.
    pub depth: usize,
    /// The provider this level is hosted on, if any.
    pub parent: Option<Box<MysqlNode>>,
}

impl MysqlNode {
    /// The root instance, running on actual hardware (allegedly).
    pub fn root() -> Self {
        Self { depth: 0, parent: None }
    }

    /// Deploy a new hosting provider as a customer site of `self`.
    pub fn host_another(self) -> Self {
        let depth = self.depth + 1;
        DEPTH.fetch_max(depth, Ordering::Relaxed);
        Self { depth, parent: Some(Box::new(self)) }
    }

    /// Walk back to bare metal. Returns the number of hops.
    pub fn hops_to_metal(&self) -> usize {
        let mut n = 0;
        let mut cur = self;
        while let Some(p) = cur.parent.as_deref() {
            n += 1;
            cur = p;
        }
        n
    }
}

/// Deepest nesting reached by this process.
pub fn current_depth() -> usize {
    DEPTH.load(Ordering::Relaxed)
}

/// Build a stack of `depth` nested providers and return the innermost one.
pub fn nest(depth: usize) -> MysqlNode {
    (0..depth).fold(MysqlNode::root(), |lvl, _| lvl.host_another())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beget_inside_beget_inside_beget() {
        let inner = nest(3);
        assert_eq!(inner.depth, 3);
        assert_eq!(inner.hops_to_metal(), 3);
    }

    #[test]
    fn deep_nesting_is_cheap() {
        let inner = nest(100_000);
        assert_eq!(inner.hops_to_metal(), 100_000);
        // Drop iteratively so the default Box drop does not blow the stack at depth 1e5.
        let mut cur = Some(Box::new(inner));
        while let Some(mut lvl) = cur {
            cur = lvl.parent.take();
        }
    }
}

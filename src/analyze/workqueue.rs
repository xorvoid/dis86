use crate::segoff::SegOff;
use std::collections::{BTreeSet, HashSet};

pub struct WorkQueue {
  discovered: HashSet<SegOff>,
  queued: BTreeSet<SegOff>,
}

impl WorkQueue {
  pub fn new() -> WorkQueue {
    WorkQueue {
      discovered: HashSet::new(),
      queued: BTreeSet::new(),
    }
  }

  pub fn insert(&mut self, addr: SegOff) {
    // Only enqueue work if its a newly discovered address
    if self.discovered.get(&addr).is_none() {
      self.discovered.insert(addr);
      self.queued.insert(addr);
    }
  }

  pub fn pop(&mut self) -> Option<SegOff> {
    self.queued.pop_first()
  }
}

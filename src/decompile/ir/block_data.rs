use super::def::*;

// Requirements:
//  - Iterate all instrs (start to finish)
//  - Get ref to last item

//  - Get the total count
//  - Get previous ref (from a ref)
//  - Get next ref (from a ref)
//  - Lookup item (from a ref)
//  - Insert front / back / before / after

pub enum Loc {
  First,
  Last,
  Before(Ref), // Ref::InstrRef
  After(Ref),  // Ref::InstrRef
}

#[derive(Debug)]
struct InstrLink {
  // Intrusive linked-list (for block)
  prev: Option<Ref>, // Ref::InstrRef (assumed to be in the same block)
  next: Option<Ref>, // Ref::InstrRef (assumed to be in the same block)
}

#[derive(Debug)]
pub struct InstrData {
  blkref: BlockRef,
  links: Vec<InstrLink>,
  instrs: Vec<Instr>,
  count: usize,
  first: Option<Ref>, // Ref::InstrRef
  last: Option<Ref>,  // Ref::InstrRef
}

impl InstrData {
  pub fn new(blkref: BlockRef) -> Self {
    Self {
      blkref,
      links: vec![],
      instrs: vec![],
      count: 0,
      first: None,
      last: None,
    }
  }

  pub fn count(&self) -> usize       { self.count }
  pub fn first(&self) -> Option<Ref> { self.first }
  pub fn last(&self)  -> Option<Ref> { self.last }

  pub fn lookup(&self, r: Ref) -> &Instr {
    let Ref::Instr(blkref, idx) = r else { panic!("expected instr ref") };
    assert!(blkref == self.blkref);
    let Some(instr) = self.instrs.get(idx) else { panic!("invalid ref") };
    instr
  }

  pub fn lookup_mut(&mut self, r: Ref) -> &mut Instr {
    let Ref::Instr(blkref, idx) = r else { panic!("expected instr ref") };
    assert!(blkref == self.blkref);
    let Some(instr) = self.instrs.get_mut(idx) else { panic!("invalid ref") };
    instr
  }

  fn lookup_link(&self, r: Ref) -> &InstrLink {
    let Ref::Instr(blkref, idx) = r else { panic!("expected instr ref") };
    assert!(blkref == self.blkref);
    let Some(link) = self.links.get(idx) else { panic!("invalid ref") };
    link
  }

  fn lookup_link_mut(&mut self, r: Ref) -> &mut InstrLink {
    let Ref::Instr(blkref, idx) = r else { panic!("expected instr ref") };
    assert!(blkref == self.blkref);
    let Some(link) = self.links.get_mut(idx) else { panic!("invalid ref") };
    link
  }

  pub fn insert(&mut self, instr: Instr, loc: Loc) -> Ref {
    let idx = self.instrs.len();
    self.instrs.push(instr);
    self.links.push(InstrLink{prev: None, next: None});
    self.count += 1;
    let r = Ref::Instr(self.blkref, idx);
    let link = &mut self.links[idx];

    match loc {
      Loc::First => {
        link.prev = None;
        link.next = self.first;
        if let Some(next) = self.first {
          let next_link = self.lookup_link_mut(next);
          next_link.prev = Some(r);
        }
        self.first = Some(r);
      }
      Loc::Last => {
        link.prev = self.last;
        link.next = None;
        if let Some(prev) = self.last {
          let prev_link = self.lookup_link_mut(prev);
          prev_link.next = Some(r);
        }
        self.last = Some(r);
      }
      Loc::Before(rref) => {
        let next = rref;
        let prev = self.prev(rref); // possibly None, if first

        self.lookup_link_mut(r).next = Some(next);
        self.lookup_link_mut(r).prev = prev;

        self.lookup_link_mut(next).prev = Some(r);

        match prev {
          Some(prev) => self.lookup_link_mut(prev).next = Some(r),
          None => self.first = Some(r),
        }
      }
      _ => panic!("UNIMPL INSERT METHOD"),
    }

    // first up first/last invariants (on first insert)
    if self.first.is_none() {
      self.first = Some(r);
    }
    if self.last.is_none() {
      self.last = Some(r);
    }

    r
  }

  pub fn next(&self, r: Ref) -> Option<Ref> {
    self.lookup_link(r).next
  }

  pub fn prev(&self, r: Ref) -> Option<Ref> {
    self.lookup_link(r).prev
  }
}

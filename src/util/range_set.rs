pub struct Range {
  pub start: u32,
  pub end: u32,
}

pub struct RangeSet(Vec<Range>);

impl RangeSet {
  pub fn new() -> Self {
    RangeSet(Vec::new())
  }

  pub fn insert(&mut self, start: u32, end: u32) {
    // Construct
    let new = Range { start, end };

    // Ignore empty
    if new.start >= new.end {
      return;
    }

    let ranges = &mut self.0;

    // First range that could overlap or be adjacent: find where end >= new.start
    let lo = ranges.partition_point(|r| r.end < new.start);

    // One past the last range that could overlap or be adjacent: start <= new.end
    let hi = ranges.partition_point(|r| r.start <= new.end);

    let merged = if lo < hi {
      Range {
        start: ranges[lo].start.min(new.start),
        end: ranges[hi - 1].end.max(new.end),
      }
    } else {
      new
    };

    ranges.splice(lo..hi, [merged]);
  }

  pub fn span(&self) -> Option<Range> {
    let ranges = &self.0;
    Some(Range {
      start: ranges.first()?.start,
      end: ranges.last()?.end,
    })
  }

  #[allow(dead_code)]
  pub fn span_start(&self) -> Option<u32> {
    Some(self.span()?.start)
  }

  pub fn span_end(&self) -> Option<u32> {
    Some(self.span()?.end)
  }

  pub fn gaps_within(&self, start: u32, end: u32) -> Vec<Range> {
    let query = Range {start, end};

    if query.start >= query.end {
      return vec![];
    }

    let ranges = &self.0;

    // Only consider ranges that overlap with the query
    let lo = ranges.partition_point(|r| r.end <= query.start);
    let hi = ranges.partition_point(|r| r.start < query.end);

    let mut gaps = Vec::new();
    let mut cursor = query.start;

    for r in &ranges[lo..hi] {
      if cursor < r.start {
        gaps.push(Range { start: cursor, end: r.start });
      }
      cursor = cursor.max(r.end);
    }

    if cursor < query.end {
      gaps.push(Range { start: cursor, end: query.end });
    }

    gaps
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ranges(rs: &RangeSet) -> Vec<(u32, u32)> {
    rs.0.iter().map(|r| (r.start, r.end)).collect()
  }

  #[test]
  fn insert_into_empty() {
    let mut rs = RangeSet::new();
    rs.insert(2, 5);
    assert_eq!(ranges(&rs), [(2, 5)]);
  }

  #[test]
  fn insert_non_overlapping_before() {
    let mut rs = RangeSet::new();
    rs.insert(5, 8);
    rs.insert(1, 3);
    assert_eq!(ranges(&rs), [(1, 3), (5, 8)]);
  }

  #[test]
  fn insert_non_overlapping_after() {
    let mut rs = RangeSet::new();
    rs.insert(1, 3);
    rs.insert(5, 8);
    assert_eq!(ranges(&rs), [(1, 3), (5, 8)]);
  }

  #[test]
  fn insert_non_overlapping_middle() {
    let mut rs = RangeSet::new();
    rs.insert(1, 3);
    rs.insert(7, 10);
    rs.insert(4, 5);
    assert_eq!(ranges(&rs), [(1, 3), (4, 5), (7, 10)]);
  }

  #[test]
  fn merge_overlapping() {
    let mut rs = RangeSet::new();
    rs.insert(1, 5);
    rs.insert(3, 8);
    assert_eq!(ranges(&rs), [(1, 8)]);
  }

  #[test]
  fn merge_adjacent() {
    let mut rs = RangeSet::new();
    rs.insert(1, 3);
    rs.insert(3, 5);
    assert_eq!(ranges(&rs), [(1, 5)]);
  }

  #[test]
  fn merge_multiple() {
    let mut rs = RangeSet::new();
    rs.insert(1, 3);
    rs.insert(7, 10);
    rs.insert(2, 8);
    assert_eq!(ranges(&rs), [(1, 10)]);
  }

  #[test]
  fn merge_adjacent_both_sides() {
    let mut rs = RangeSet::new();
    rs.insert(1, 3);
    rs.insert(5, 7);
    rs.insert(3, 5);
    assert_eq!(ranges(&rs), [(1, 7)]);
  }

  #[test]
  fn insert_superset() {
    let mut rs = RangeSet::new();
    rs.insert(3, 5);
    rs.insert(1, 8);
    assert_eq!(ranges(&rs), [(1, 8)]);
  }

  #[test]
  fn insert_subset() {
    let mut rs = RangeSet::new();
    rs.insert(1, 8);
    rs.insert(3, 5);
    assert_eq!(ranges(&rs), [(1, 8)]);
  }

  #[test]
  fn span_empty() {
    let rs = RangeSet::new();
    assert!(rs.span().is_none());
  }

  #[test]
  fn span_single() {
    let mut rs = RangeSet::new();
    rs.insert(3, 7);
    let s = rs.span().unwrap();
    assert_eq!((s.start, s.end), (3, 7));
  }

  #[test]
  fn span_multiple() {
    let mut rs = RangeSet::new();
    rs.insert(3, 7);
    rs.insert(10, 20);
    rs.insert(50, 60);
    let s = rs.span().unwrap();
    assert_eq!((s.start, s.end), (3, 60));
  }

  #[test]
  fn gaps_within_no_ranges() {
    let rs = RangeSet::new();
    let gaps: Vec<_> = rs.gaps_within(0, 100).into_iter().map(|r| (r.start, r.end)).collect();
    assert_eq!(gaps, [(0, 100)]);
  }

  #[test]
  fn gaps_within_basic() {
    let mut rs = RangeSet::new();
    rs.insert(10, 20);
    rs.insert(50, 60);
    let gaps: Vec<_> = rs.gaps_within(0, 100).into_iter().map(|r| (r.start, r.end)).collect();
    assert_eq!(gaps, [(0, 10), (20, 50), (60, 100)]);
  }

  #[test]
  fn gaps_within_fully_covered() {
    let mut rs = RangeSet::new();
    rs.insert(0, 100);
    let gaps: Vec<_> = rs.gaps_within(0, 100).into_iter().map(|r| (r.start, r.end)).collect();
    assert_eq!(gaps, []);
  }

  #[test]
  fn gaps_within_query_inside_range() {
    let mut rs = RangeSet::new();
    rs.insert(0, 100);
    let gaps: Vec<_> = rs.gaps_within(20, 50).into_iter().map(|r| (r.start, r.end)).collect();
    assert_eq!(gaps, []);
  }

  #[test]
  fn gaps_within_ranges_outside_query_ignored() {
    let mut rs = RangeSet::new();
    rs.insert(0, 5);
    rs.insert(40, 50);
    rs.insert(95, 100);
    let gaps: Vec<_> = rs.gaps_within(10, 90).into_iter().map(|r| (r.start, r.end)).collect();
    assert_eq!(gaps, [(10, 40), (50, 90)]);
  }

  #[test]
  fn gaps_within_empty_query() {
    let mut rs = RangeSet::new();
    rs.insert(10, 20);
    let gaps = rs.gaps_within(5, 5);
    assert_eq!(gaps.len(), 0);
  }

  #[test]
  fn empty_range_ignored() {
    let mut rs = RangeSet::new();
    rs.insert(3, 3);
    rs.insert(5, 2);
    assert_eq!(ranges(&rs), []);
  }
}

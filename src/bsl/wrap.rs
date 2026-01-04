use crate::bsl::bind::*;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::marker::PhantomData;

pub enum Value<'a> {
  Node(Node<'a>),
  Str(&'a str),
}

#[derive(Clone)]
pub struct Node<'a> {
  ctx: *mut bsl_t,
  phantom: PhantomData<&'a Root>,
}

pub struct Root {
  ctx: *mut bsl_t,
}

pub struct Iter<'a> {
  it: bsl_iter_t,
  phantom: PhantomData<Node<'a>>,
}

pub fn parse(inp: &str) -> Option<Root> {
  Root::parse(inp)
}

impl Root {
  pub fn parse(inp: &str) -> Option<Root> {
    let mut err: c_int = 0;
    let ctx = unsafe { bsl_parse_new(inp.as_ptr() as *const _, inp.len(), &mut err) };
    if ctx.is_null() {
      return None
    }

    Some(Root { ctx })
  }

  #[allow(unused)]
  pub fn get(&self, key: &str) -> Option<Value<'_>> { node_get(self.ctx, key) }
  #[allow(unused)]
  pub fn get_str(&self, key: &str) -> Option<&str> { node_get_str(self.ctx, key) }
  #[allow(unused)]
  pub fn get_node(&self, key: &str) -> Option<Node<'_>> { node_get_node(self.ctx, key) }
}

impl Drop for Root {
  fn drop(&mut self) {
    unsafe { bsl_delete(self.ctx) };
  }
}

impl<'a> Value<'a> {
  fn from_raw(typ: c_int, ptr: *mut c_void) -> Value<'a> {
    if typ == BSL_TYPE_STR {
      let cstr = unsafe { CStr::from_ptr(ptr as *const c_char) };
      Value::Str(cstr.to_str().unwrap())
    } else if typ == BSL_TYPE_NODE {
      Value::Node(Node { ctx: ptr as *mut bsl_t, phantom: PhantomData})
    } else {
      panic!("Unexpected type: {}", typ);
    }
  }

  pub fn as_node(&self) -> Option<Node<'a>> {
    match self {
      Self::Node(n) => Some(n.clone()),
      _ => None,
    }
  }

  #[allow(unused)]
  pub fn as_str(&self) -> Option<&'a str> {
    match self {
      Self::Str(s) => Some(s),
      _ => None,
    }
  }
}

fn node_get<'a>(ctx: *mut bsl_t, key: &str) -> Option<Value<'a>> {
  let key_cstr = CString::new(key).unwrap();
  let mut typ: c_int = 0;
  let ptr = unsafe { bsl_get_generic(ctx, key_cstr.as_ptr(), &mut typ) };
  if ptr.is_null() {
    return None;
  }
  Some(Value::from_raw(typ, ptr))
}

fn node_get_str<'a>(ctx: *mut bsl_t, key: &str) -> Option<&'a str> {
  let elt = node_get(ctx, key)?;
  match elt {
    Value::Str(s) => Some(s),
    _ => None,
  }
}

fn node_get_node<'a>(ctx: *mut bsl_t, key: &str) -> Option<Node<'a>> {
  let elt = node_get(ctx, key)?;
  match elt {
    Value::Node(n) => Some(n),
    _ => None,
  }
}

impl<'a> Node<'a> {
  #[allow(unused)]
  pub fn get(&self, key: &str) -> Option<Value<'_>> { node_get(self.ctx, key) }
  #[allow(unused)]
  pub fn get_str(&self, key: &str) -> Option<&str> { node_get_str(self.ctx, key) }
  #[allow(unused)]
  pub fn get_node(&self, key: &str) -> Option<Node<'_>> { node_get_node(self.ctx, key) }

  pub fn iter(&self) -> Iter<'_> {
    let mut it: bsl_iter_t = unsafe { MaybeUninit::zeroed().assume_init() };
    unsafe { bsl_iter_begin(&mut it, self.ctx) };
    Iter { it, phantom: PhantomData }
  }
}

impl<'a> Iterator for Iter<'a> {
  type Item = (&'a str, Value<'a>);
  fn next(&mut self) -> Option<Self::Item> {
    let mut typ: c_int = -1;
    let mut key: *const c_char = std::ptr::null();
    let mut val: *mut c_void = std::ptr::null_mut();
    let valid = unsafe { bsl_iter_next(&mut self.it, &mut typ, &mut key, &mut val) };
    if !valid {
      return None;
    }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap();
    let elt = Value::from_raw(typ, val);
    Some((key, elt))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_1() {
    let inp = "foo bar";
    let root = Root::parse(inp).unwrap();
    assert_eq!(root.get_str("foo"), Some("bar"));
    assert_eq!(root.get_str("foo1"), None);
  }

  #[test]
  fn test_2() {
    let inp = "foo bar good stuff   ";
    let root = Root::parse(inp).unwrap();
    assert_eq!(root.get_str("foo"), Some("bar"));
    assert_eq!(root.get_str("good"), Some("stuff"));
    assert_eq!(root.get_str("foo1"), None);
  }

  #[test]
  fn test_3() {
    let inp = "top {foo bar baz {} } top2 r ";
    let root = Root::parse(inp).unwrap();
    assert_eq!(root.get_str("top.foo"), Some("bar"));
    assert_eq!(root.get_str("top.foo.baz"), None);
    assert!(root.get_node("top.baz").is_some());
  }

  #[test]
  fn test_4() {
    let inp = "top \"foo bar\" bot g quote \"{ key val }\"";
    let root = Root::parse(inp).unwrap();
    assert_eq!(root.get_str("top"), Some("foo bar"));
    assert_eq!(root.get_str("bot"), Some("g"));
    assert_eq!(root.get_str("quote"), Some("{ key val }"));
  }

  #[test]
  fn test_5() {
    let inp = "top { a b c { d e } }";
    let root = Root::parse(inp).unwrap();
    let top = root.get_node("top").unwrap();

    let keys: Vec<_> = top.iter().map(|(k,_)| k).collect();
    assert_eq!(keys, vec!["a", "c"]);
  }
}

use crate::types::*;

// FIXME: Unify this and the ast code

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathAccess {
  Array(usize),
  Struct(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Access {
  // Access path
  pub path: Vec<PathAccess>,

  // After applying access path
  pub off: usize,
  pub sz: usize,
  pub typ: Type,
}

fn determine_path_recurse(mut path: Vec<PathAccess>, types: &TypeDatabase, typ: &Type, mut access_off: usize, access_sz: usize) ->  Access {
  if typ.is_primitive() {
    return Access {
      path,
      off: access_off,
      sz: access_sz,
      typ: typ.clone(),
    };
  }

  match typ {
    Type::Array(basetype, len) => {
      let ArraySize::Known(len) = len else { panic!("Expected datatype to have known array length") };
      let basetype_sz = basetype.size_in_bytes().unwrap();
      let idx = access_off as usize / basetype_sz;
      if idx > *len { panic!("Access out of range"); }
      if access_sz as usize > basetype_sz { panic!("Access exceeds basetype size"); }

      path.push(PathAccess::Array(idx));
      access_off -= idx * basetype_sz;

      return determine_path_recurse(path, types, basetype, access_off, access_sz);
    }
    Type::Struct(struct_ref) => {
      let access_start = access_off;
      let access_end = access_off + access_sz;
      let s = types.lookup_struct(*struct_ref).unwrap();
      for mbr in &s.members {
        let mbr_start = mbr.off as usize;
        let mbr_end = mbr_start + mbr.typ.size_in_bytes().unwrap();
        if !(mbr_start <= access_start && access_end <= mbr_end) { continue; }

        path.push(PathAccess::Struct(mbr.name.clone()));
        access_off -= mbr.off as usize;

        return determine_path_recurse(path, types, &mbr.typ, access_off, access_sz);
      }
      panic!("Failed to find member");
    }
    _ => {
      panic!("Unknown ... {:?}", typ);
    }
  }
}

pub fn from_type_and_offset(types: &TypeDatabase, typ: &Type, off: usize, sz: usize) -> Access {
  determine_path_recurse(vec![], types, typ, off, sz)
}

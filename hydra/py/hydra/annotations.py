_TypeSizes = {
    'u8':  1,
    'i8':  1,
    'u16': 2,
    'i16': 2,
    'u32': 4,
    'i32': 4,
}

def basetype_size_in_bytes(typename):
    return _TypeSizes.get(typename, None)

class Type:
    def __init__(self, basetype, is_array, array_len):
        self.basetype = basetype
        self.is_array = is_array
        self.array_len = array_len

    @staticmethod
    def from_str(s):
        parts = s.split('[')
        if len(parts) > 2:
            raise Exception(f'Invalid type: "{s}"')

        typ = Type(parts[0], False, None)
        if len(parts) > 1:
            typ.is_array = True
            left = parts[1]
            if len(left) == 0 or left[-1] != ']':
                raise Exception('Expected closing brace in array type')
            size_str = left[:-1]
            if len(size_str) > 0:
                typ.array_len = int(size_str)
            else:
                typ.array_len = None
        return typ

    def as_basetype(self):
        return Type(self.basetype, False, None)

    def get_ctype_str_parts(self):
        assert isinstance(self.basetype, str)

        if not self.is_array:
            return (self.basetype, '')
        else:
            if self.array_len is None:
                raise Exception(f'No array length provided for: {self} ... required by get_ctype_str_parts()')

            return (self.basetype, f'[{self.array_len}]')

    def fmt_ctype_str(self, name):
        start, end = self.get_ctype_str_parts()
        return f'{start:<15} {name}{end}'

    def size_in_bytes(self):
        basesz = basetype_size_in_bytes(self.basetype)
        if not self.is_array: return basesz


        if basesz is None: return None
        if self.array_len is None: return None

        return basesz * self.array_len

    def __str__(self):
        s = self.basetype
        if self.is_array:
            sz = '' if self.array_len is None else str(self.array_len)
            s += f'[{sz}]'
        return s

class Off:
    def __init__(self, off):
        self.off = int(off, 16)
        assert 0 <= self.off and self.off < (1<<16)

    def __str__(self):
        return f'0x{self.off:04x}'

class Addr:
    def __init__(self, addr):
        parts = addr.split(':')
        if len(parts) != 2: raise Exception(f'Invalid address: "{addr}"')

        seg_str = parts[0]
        off_str = parts[1]

        self.overlay = False
        if seg_str.startswith('overlay_'):
            self.overlay = True
            seg_str = seg_str[8:]

        self.seg = int(seg_str, 16)
        self.off = int(off_str, 16)

    def bytes_since(self, start):
        assert self.overlay == start.overlay
        assert self.seg == start.seg
        return self.off - start.off

    def __str__(self):
        pre = 'overlay_' if self.overlay else ''
        return pre + f'{self.seg:04x}:{self.off:04x}'

### FIXME RENAME
UNKNOWN=-1

FUNCTION_ALL_NAMES = set()
def _verify_unique(name):
    if name in FUNCTION_ALL_NAMES:
        raise Exception(f'Duplicate function name: {name}')
    FUNCTION_ALL_NAMES.add(name)

class Function:
    def __init__(self, reimpl, name, ret, args, start_addr, end_addr, flags=0, regargs=None, entry=None):
        _verify_unique(name);
        self.name = name
        self.ret = ret
        self.args = args
        self.start_addr = Addr(start_addr)
        self.end_addr = Addr(end_addr) if end_addr else None
        self.entry_stub = Addr(entry) if entry else None
        self.is_overlay_entry = self.start_addr.overlay and self.entry_stub is not None
        self.regargs = ','.join(regargs) if regargs else None
        self.flags = flags
        self.reimpl = reimpl

class Global:
    def __init__(self, name, typ, off, flags=''):
        self.name = name
        self.typ = Type.from_str(typ)
        self.off = off
        self.flags = flags

def validate_data_section(ds):
    mem = [0] * (1<<16)
    for g in ds:
        if g.flags == 'SKIP_VALIDATE':
            continue
        off = g.off
        sz = g.typ.size_in_bytes()
        if sz is None:
            print('WARN: Cannot determine size for %s' % g.name)
            continue
        for i in range(off, off+sz):
            if mem[i] != 0:
                raise Exception('Overlap detect in %s: [0x%04x, 0x%04x]' % (g.name, off, off+sz))
            mem[i] = 1

class TextData:
    def __init__(self, name, typ, start_addr, end_addr, access_at=None):
        self.name = name
        self.typ = Type.from_str(typ)
        self.start_addr = Addr(start_addr)
        self.end_addr = Addr(end_addr)
        self.access_at = access_at

        ## infer array size
        nbytes = self.end_addr.bytes_since(self.start_addr)
        if nbytes < 0: raise Exception(f"Negatively sized text-section region: {name}")
        if not self.typ.is_array: raise Exception(f"Expected array for text-section region: {name}")
        eltsz = self.typ.as_basetype().size_in_bytes()
        if nbytes % eltsz != 0: raise Exception(f"Expected text-section region to be a multiple of {eltsz}: {name}")
        array_len = nbytes // eltsz

        ## use it or verify
        if self.typ.array_len is not None:
            if self.typ.array_len != array_len: raise Exception(f"Misconfiguration: config specified {self.typ.array_len} elements, but region contains {array_len}: {name}")
        self.typ.array_len = array_len

CALLSTACK_CONF_VALID_TYPES = { 'HANDLER', 'IGNORE_ADDR', 'JUMPRET', }

class Struct:
    def __init__(self, name, size, members):
        self.name = name
        self.size = size
        self.members = members

        if not self.name.endswith('_t'):
            raise Exception(f'Struct names should end with _t: {name}')

        if name in _TypeSizes:
            raise Exception(f'Type name has already been defined: {name}')
        _TypeSizes[name] = size

        ## validate
        off = 0
        for mbr in self.members:
            if mbr.off > off:
                raise Exception(f'Skipped bytes range {off}-{mbr.off} in struct "{name}"')
            if mbr.off < off:
                raise Exception(f'Overlapping byte range {mbr.off}-{off} in struct "{name}"')
            sz = mbr.size_in_bytes()
            if sz is None:
                raise Exception(f'Member in struct has no known size: {name}.{mbr.name}')
            off += mbr.size_in_bytes()

        if off != self.size:
            raise Exception(f'Size mismtch: struct size is {self.size} but members use {off} bytes')

    def struct_name(self):
        return self.name[:-2]

class Member:
    def __init__(self, name, typ, off):
        self.name = name
        self.typ  = Type.from_str(typ)
        self.off  = off

    def size_in_bytes(self):
        return self.typ.size_in_bytes()

class CallstackConf:
    def __init__(self, name, typ, addr):
        if typ not in CALLSTACK_CONF_VALID_TYPES:
            raise Exception(f'Not a valid callstack conf type: {typ}')
        self.name = name
        self.typ  = typ
        self.addr = Addr(addr)

class CodeSegment:
    def __init__(self, seg, name):
        self.seg = seg
        self.name = name

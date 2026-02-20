import sys

## MUST BE SET BY SETUP!
out = None
def emit(s): print(s, file=out)

def gen_code_segments(code_segments):
    emit("  code_segments {")
    for i, cs in enumerate(code_segments):
        emit(f"    _{i:04} {{ seg {cs.seg} name {cs.name:15}  }}")
    emit("  }")

def gen_functions(functions):
    emit("  functions {");
    for func in functions:
        extra = ''
        if func.flags == 'DONT_POP_ARGS':
            extra += 'dont_pop_args 1 '
        if func.flags == 'INDIRECT_CALL_LOCATION':
            extra += 'indirect_call_location 1 '
        if func.entry_stub is not None:
            extra += f'entry {func.entry_stub} '
        if func.regargs is not None:
            extra += f'regargs {func.regargs} '
        mode = 'far'
        if func.flags == 'NEAR':
            mode = 'near'
        start = '""' if not func.start_addr else func.start_addr
        end = '""' if not func.end_addr else func.end_addr
        emit(f'    {func.name:30} {{ start {start} end {end} mode {mode} ret {func.ret} args {func.args} {extra}}} ')
    emit("  }");

def gen_structures(structures):
    emit("  structures {")
    for struct in structures:
        emit(f"    {struct.name:<15} {{ size {struct.size} members {{")
        for i, mbr in enumerate(struct.members):
            num = f'_{i}'
            emit(f"      {mbr.name:<20} {{ type {str(mbr.typ):<15} off 0x{mbr.off:02x} }}")
        emit(f"    }}}}")
    emit("  }")

def gen_data_section(datasection):
    emit("  globals {")
    for var in datasection:
        emit(f'    {var.name:30} {{ off 0x{var.off:04x}  type {str(var.typ):20} }}')
    emit("  }");

def gen_text_section(textsection):
    emit("  text_section {");
    for var in textsection:
        extra = ''
        if var.access_at is not None:
            extra += f'access {var.access_at} '
        emit(f'    {var.name:30} {{ start {var.start_addr}  end {var.end_addr} type {str(var.typ):20} {extra}}}')
    emit("  }");

# def gen_segmap():
#     emit("  segmap {");
#     emit("    _00  { from 0000 to 0000 }");
#     emit("    _01  { from 0008 to 02e0 }");
#     emit("    _02  { from 0010 to 0399 }");
#     emit("    _03  { from 0018 to 0454 }");
#     emit("    _04  { from 0020 to 0473 }");
#     emit("    _05  { from 0028 to 04ff }");
#     emit("    _06  { from 0030 to 052c }");
#     emit("    _07  { from 0038 to 0536 }");
#     emit("    _08  { from 0040 to 053f }");
#     emit("    _09  { from 0048 to 0581 }");
#     emit("    _10  { from 0050 to 05df }");
#     emit("    _11  { from 0058 to 0622 }");
#     emit("    _12  { from 0060 to 074b }");
#     emit("    _13  { from 0068 to 07a0 }");
#     emit("    _14  { from 0070 to 07ab }");
#     emit("    _15  { from 0078 to 0834 }");
#     emit("    _16  { from 0080 to 08f5 }");
#     emit("    _17  { from 0088 to 098d }");
#     emit("    _18  { from 0090 to 09c0 }");
#     emit("    _19  { from 0098 to 0a04 }");
#     emit("    _20  { from 00a0 to 0adb }");
#     emit("    _21  { from 00a8 to 0b48 }");
#     emit("    _22  { from 00b0 to 0bb4 }");
#     emit("    _23  { from 00b8 to 0cdd }");
#     emit("    _24  { from 00c0 to 0d3c }");
#     emit("    _25  { from 00c8 to 0d42 }");
#     emit("    _26  { from 00d0 to 0dd7 }");
#     emit("    _27  { from 00d8 to 0dd7 }");
#     emit("    _28  { from 00e0 to 0dd7 }");
#     emit("    _29  { from 00e8 to 0dd7 }");
#     emit("    _30  { from 00f0 to 0de1 }");
#     emit("    _31  { from 00f8 to 0de3 }");
#     emit("    _32  { from 0100 to 0de3 }");
#     emit("    _33  { from 0108 to 0de3 }");
#     emit("    _34  { from 0110 to 0de3 }");
#     emit("    _35  { from 0118 to 0e0d }");
#     emit("    _36  { from 0120 to 0e0d }");
#     emit("    _37  { from 0128 to 0e13 }");
#     emit("    _38  { from 0130 to 0e18 }");
#     emit("    _39  { from 0138 to 0e1d }");
#     emit("    _40  { from 0140 to 0e20 }");
#     emit("    _41  { from 0148 to 0e25 }");
#     emit("    _42  { from 0150 to 0e28 }");
#     emit("    _43  { from 0158 to 0e2d }");
#     emit("    _44  { from 0160 to 0e30 }");
#     emit("    _45  { from 0168 to 0e34 }");
#     emit("    _46  { from 0170 to 0e36 }");
#     emit("    _47  { from 0178 to 0e3a }");
#     emit("    _48  { from 0180 to 0e3d }");
#     emit("    _49  { from 0188 to 0e41 }");
#     emit("    _50  { from 0190 to 0e46 }");
#     emit("    _51  { from 0198 to 0e4e }");
#     emit("    _52  { from 01a0 to 0e51 }");
#     emit("    _53  { from 01a8 to 0e5d }");
#     emit("    _54  { from 01b0 to 0e60 }");
#     emit("    _55  { from 01b8 to 0e65 }");
#     emit("    _56  { from 01c0 to 0e6a }");
#     emit("    _57  { from 01c8 to 0e6d }");
#     emit("    _58  { from 01d0 to 0e70 }");
#     emit("    _59  { from 01d8 to 0e73 }");
#     emit("    _60  { from 01e0 to 0e7a }");
#     emit("    _61  { from 01e8 to 0e7d }");
#     emit("    _62  { from 01f0 to 0e80 }");
#     emit("    _63  { from 01f8 to 0e83 }");
#     emit("    _64  { from 0200 to 0e88 }");
#     emit("    _65  { from 0208 to 0e8e }");
#     emit("    _66  { from 0210 to 0e90 }");
#     emit("    _67  { from 0218 to 0e94 }");
#     emit("    _68  { from 0220 to 0e97 }");
#     emit("    _69  { from 0228 to 0e9b }");
#     emit("    _70  { from 0230 to 0e9f }");
#     emit("  }");

def gen_conf(data, outfile=None):
    global out
    out = sys.stdout if not outfile else outfile
    emit("dis86 {");
    gen_code_segments(data['code_segments'])
    gen_functions(data['functions'])
    gen_structures(data['structures'])
    gen_data_section(data['data_section'])
    gen_text_section(data['text_section'])
    #gen_segmap()
    emit("}");

#!/usr/bin/env python
import fileinput
import re
from string import Template

find_real_func = re.compile('\s* fn \s* (\w+) \( ([^)]+) \)(?: \s* -> \s* ErrnoResult \s* < ([^>]+) >)? \s* \{ \s* fail!\(\) \s* \}', re.X)

funcs = []

for line in fileinput.input():
    match = find_real_func.match(line)
    if not match:
        continue
    fn_name = match.group(1)
    fn_args = match.group(2)
    result_type = match.group(3)
    funcs.append([fn_name, fn_args, result_type])

print """
pub fn make_fuse_ll_oper<Ops:FuseLowLevelOps>(ops:&Ops)
    -> fuse::Struct_fuse_lowlevel_ops {
    return fuse::Struct_fuse_lowlevel_ops {"""
for (name, args, result) in funcs:
    if re.match("_size$", name):
        continue
    print Template("        ${name}: if ops.${name}_is_implemented() { ${name}_impl } else { ptr::null() },").substitute(name=name)

print """
    }
}

extern fn init_impl(userdata:*c_void, conn:*fuse::Struct_fuse_conn_info) {
    userdata_to_ops(userdata).init();
}

extern fn destroy_impl(userdata:*c_void) {
    userdata_to_ops(userdata).destroy();
}"""

for (name, args, result) in funcs:
    if name == "init" or name == "destroy":
        continue
    print Template("""
extern fn ${name}_impl() { fail!() }""").substitute(name=name)

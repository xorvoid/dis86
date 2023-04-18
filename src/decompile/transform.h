#pragma once

// xor r,r => mov r,0
void transform_pass_xor_rr(meh_t *m);

// cmp a,b; j{pred} target => {c-style code}
void transform_pass_cmp_jmp(meh_t *m);

// or r,r; j{e|ne} target => {c-style code}
void transform_pass_or_jmp(meh_t *m);

// synthesize normal calls where possible
void transform_pass_synthesize_calls(meh_t *m);

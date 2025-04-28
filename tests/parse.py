#!/bin/env python3

import subprocess
import re

disassembly = subprocess.check_output(
    ["llvm-objdump-18", "-d", "encode.o", "--no-print-imm-hex"]
).splitlines()

for line in disassembly:
    # Some llvm-objdump output examples:
    #     1168: fa8c fa8b      qadd    r10, r11, r12
    #     1132: e7f7           b       0x1124 <label_b>        @ imm = #-0x12
    #     10d0: bf00           nop
    # We have an unused capture group to remove symbols resolution ("<.text+0xaa>" for
    # instance).
    # Last capture group is for catching eventual end of line comments starting with ";".
    # We don't want to output the comments.
    m = re.match(
        r"^\s*[0-9a-f]+:\s*(?:([0-9a-f]{4}) ?([0-9a-f]{4})?)\s*(\S+)\t?([^;<@]*)(@.*)?(<.*>\s*)?(;.*)?$",
        #     --------- address                                ----- name             ---------- discard symbol
        #                  --------------------------------- encoding  --------- args
        line.decode(),
    )
    if m is not None:
        hw1 = bytes.fromhex(m.groups()[0])
        data = bytearray([hw1[1], hw1[0]])
        if m.groups()[1] is not None:
            hw2 = bytes.fromhex(m.groups()[1])
            data += bytes([hw2[1], hw2[0]])
        op = m.groups()[2]
        args = m.groups()[3].lower().strip()
        # strip() required when instruction has no arguments
        vector = f"{data.hex():<8} {op:<9}{args}".strip()
        print(vector)

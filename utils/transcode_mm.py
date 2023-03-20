#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import scipy.io
import sys
from io import BytesIO

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(f"Usage: {sys.argv[0]} <original_mm_file> <transcoded_mm_file>")

    eprint(f"Reading MatrixMarket file {sys.argv[1]}...")
    mmfile = scipy.io.mmread(str(sys.argv[1]))

    eprint(f"Writing to MatrixMarket file {sys.argv[2]}...")
    if str(sys.argv[2]) == "stdout":
        target = BytesIO()
    else:
        target = str(sys.argv[2])

    scipy.io.mmwrite(target, mmfile, comment='\n This file was generated with transcode_mm.py tool from matrix_rs.\n')

    if str(sys.argv[2]) == "stdout":
        print(target.getvalue().decode('utf8'))

    eprint("Done")
#!/usr/bin/env python3
from pathlib import Path
import hashlib
import sys

from python_reference import build_core_minimal

root = Path(__file__).resolve().parents[1]
golden = (root / "fixtures/golden/core-minimal.vfm").read_bytes()
independent, info = build_core_minimal()
if independent != golden:
    print("FAIL: independent Python implementation differs from golden", file=sys.stderr)
    sys.exit(1)
print("OK byte-identical")
print("size", len(golden))
print("sha256", hashlib.sha256(golden).hexdigest())
print("content_root_digest", info["content_root_digest"])

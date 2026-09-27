# VFM 1.3 — Core Freeze Candidate

VFM (Vietflex Map Format) 1.3 freezes the **binary Core** before expanding domain profiles. The Core is intentionally small: container structure, identity, byte addressing, integrity, profile discovery and deterministic conformance.

> **Freeze the core. Extend by profile.**

## What is real in this repository

This repository includes an executable conformance seed, not only a design document:

- `fixtures/golden/core-minimal.vfm` — a real, deterministic **584-byte VFM 1.3 artifact** containing the required `META`, `HASH` and `PROF` sections.
- `fixtures/golden/core-chunked.vfm` — a real chunked artifact exercising the frozen 128-byte ChunkDescriptor.
- `src/lib.rs` — Rust reference reader/writer for the frozen Core structures.
- `src/bin/vfm.rs` — minimal CLI (`build-minimal`, `validate`, `inspect`).
- `tools/python_reference.py` — an **independent Python generator** that does not call the Rust implementation.
- `tests/conformance.rs` — byte identity + malformed corpus tests.
- `fixtures/malformed/` — fixtures targeting stable VFM error codes.
- `index.html` — VFM 1.3 Core Binary Specification website.
- `spec/v1.3/core-binary.md` — normative text form of the frozen wire contract.

The central interoperability proof is reproducible:

```text
VFM 1.3 byte-level specification
          │
    ┌─────┴─────┐
    ▼           ▼
Rust writer   Python writer
    │           │
    └─────┬─────┘
          ▼
  byte-for-byte cmp
          │
          ▼
fixtures/golden/core-minimal.vfm
SHA-256: 5b65b13e2ae03aa3697dc869edf5168e6601be7465b1eba10331c145ebaaeefc
```

## Frozen Core 1.3 constants

| Item | Value |
|---|---|
| Magic | `56 46 4D 00 0D 0A 1A 0A` |
| Byte order | little-endian only |
| Header | 256 bytes |
| Root Directory Entry | 64 bytes |
| Chunk Descriptor | 128 bytes |
| HashRecord | 48 bytes |
| Root Directory offset | 256 |
| Minimum alignment | 8 bytes |
| Content hash | SHA-256 |
| Stored-byte corruption check | CRC32C |
| META/PROF encoding | deterministic CBOR |

Domain semantics such as Feature, Raster, 3D, Temporal, BIM, Sensor and GeoAI remain **outside the frozen Core** and evolve through profiles.

## Build and validate

```bash
cargo test --all-targets
cargo run --bin vfm -- build-minimal /tmp/core-minimal.vfm
cmp /tmp/core-minimal.vfm fixtures/golden/core-minimal.vfm
cargo run --bin vfm -- validate fixtures/golden/core-minimal.vfm
cargo run --bin vfm -- validate fixtures/golden/core-chunked.vfm
cargo run --bin vfm -- inspect fixtures/golden/core-minimal.vfm
```

Independent Python implementation:

```bash
python tools/python_reference.py --output /tmp/python-core-minimal.vfm
cmp /tmp/python-core-minimal.vfm fixtures/golden/core-minimal.vfm
python tools/verify_independent.py
```

Regenerate the fixture corpus:

```bash
python tools/python_reference.py --fixtures-root .
```

## Conformance targets

The current seed covers:

- frozen 256-byte Header and 64-byte Directory Entry
- frozen 128-byte ChunkDescriptor
- deterministic byte output for `core-minimal.vfm`
- Rust reader/writer and independent Python writer byte identity
- CRC32C + SHA-256 integrity chain
- stable failures for bad magic, Header CRC, reserved bits, Directory digest, unaligned offsets, overlapping ranges, invalid hash references, content hash mismatch and root digest mismatch
- GitHub Actions CI that reproduces and compares the binary artifact

Fast-Open, optional compression and additional domain-profile golden vectors are later conformance expansions; they must not change the frozen Core 1.x structs.

## Website

`index.html` is a standalone VFM 1.3 specification site suitable for GitHub Pages. A Pages workflow is included. If Pages is not already enabled for this repository, select **Settings → Pages → Source: GitHub Actions** once.

## Status

**Core Freeze Candidate**, not an OGC/ISO/IETF standard. Draft 1.0–1.2 were exploratory and are not byte-stable compatibility targets.

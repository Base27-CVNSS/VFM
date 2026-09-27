# VFM 1.3 — Cross-GIS Protocol / Core Freeze Candidate

**VFM (Vietflex Map Format)** is positioned first as a **cross-GIS interoperability protocol** for heterogeneous spatial datasets.

Its purpose is not to replace GeoJSON, GeoPackage, GeoParquet, COG, LAS/LAZ, IFC, CityGML, 3D Tiles, sensor APIs or other mature domain formats. VFM defines the common contract that lets those datasets identify and relate the **same physical or logical world** across systems.

> **VFM standardizes how spatial datasets understand each other — not how every dataset must be physically stored.**

The `.vfm` binary artifact is one normative encoding/container of that protocol.

```text
VFM Protocol
    |
    +-- Logical interoperability
    |     identity
    |     spatial reference / frames
    |     time / state
    |     relations / topology
    |     semantics bindings
    |     provenance
    |     integrity
    |
    +-- Domain profiles
    |     Feature / Raster / Point Cloud / 3D / BIM
    |     Temporal / Sensor / GeoAI / Digital Twin / Robotics
    |
    +-- Encodings / transports
          .vfm binary
          API / stream
          database representation
          future serializations
```

Therefore:

```text
VFM != .vfm
.vfm = one encoding of the VFM Protocol
```

## Why this matters

The root problem in spatial digital transformation is not simply "too many file formats". The deeper problem is that multiple systems can describe the same real-world entity without a stable, machine-verifiable way to know that they refer to the same thing, in the same space and time.

A bridge may exist as a GIS feature, an IFC object, a LiDAR segment, a drone observation, a sensor asset and a Digital Twin object. VFM should make those representations interoperable without forcing destructive conversion into one storage format.

```text
GeoJSON / GeoParquet / GPKG ----\
COG / imagery -------------------\
LAS / LAZ ------------------------> VFM Protocol ---> WebGIS
IFC / BIM -----------------------/                  BIM
3D Tiles / CityGML -------------/                   Digital Twin
Sensors / SLAM / trajectories --/                    GeoAI / Robot
```

The core interoperability primitives are:

- **stable identity** across native IDs;
- **CRS / reference frames / units / transforms**;
- **time and state**;
- **relations and topology**;
- **semantic bindings** without rewriting native schemas;
- **provenance and transformation history**;
- **content integrity and deterministic identity**.

See: [VFM 1.3 Cross-GIS Protocol Model](spec/v1.3/cross-gis-protocol.md).


## Operational scope: what VFM does and does not do

VFM is intentionally **not** a data-capture tool, GIS editor or spatial-analysis engine. Its durable role is the data layer between producers and consumers.

```text
Source systems
survey / GIS / BIM / IoT / imagery / databases
        |
        v
VFM
normalize -> convert -> package -> index -> protect
         -> distribute -> visualize -> preserve
        |
        v
Consumer systems
WebGIS / GIS / BIM / Digital Twin / AI / archive
```

VFM SHOULD focus on:

- **digital conversion and normalization** between heterogeneous datasets;
- **containerization, chunking, indexing and LOD** for partial/random access;
- **stable identity, time, spatial reference and relationships**;
- **integrity, signing and optional encryption**;
- **distribution and cache-friendly delivery**;
- **lightweight visualization profiles** for inspection and WebGIS;
- **versioning, provenance and long-term preservation**.

VFM does **not** define field acquisition, digitizing/editing workflows, cartographic authoring, buffer/intersection/network analysis or general GIS analytics. Those remain the responsibility of GIS/BIM/GeoAI tools.

## WebGIS reference stack: thin, stable, replaceable

WebGIS display is the first operational reference path for VFM, but the renderer and storage backend are deliberately kept outside the frozen Core.

```text
ONE DATASET
    |
    v
dataset.vfmm               <- one logical entry point
    |
    v
VFM Web Profile
    |
    +--> PMTiles / MVT     <- reference delivery profile
    |
    v
HTTP Range / cache
    |
    v
R2 / S3-compatible object storage
    |
    v
MapLibre GL JS             <- reference renderer
    |
    v
WebGIS
```

The reference implementation principle is:

> **One Dataset -> One Manifest -> One Runtime API -> One Data Domain.**

MapLibre GL JS is a **reference renderer**, not part of VFM identity. Cloudflare R2 is a **reference deployment backend**, not a required VFM service. PMTiles/MVT is the recommended WebGIS delivery profile so the browser does not need to decode the entire canonical VFM dataset.

The Web runtime contract SHOULD require only standard web capabilities such as HTTPS GET, byte-range requests, CORS, cache metadata and immutable/versioned assets.

Two architectural invariants are recommended:

> **A VFM dataset MUST remain renderable without blockchain, IPFS, AI, MCP or a GIS server.**

> **The VFM Web Profile SHOULD be renderable by a standard MapLibre client from static HTTP-compatible object storage.**

## Optional trust and decentralized distribution

IPFS, content-addressed storage and blockchain are future/optional adapters, not the WebGIS critical path.

```text
                     VFM
                      |
             +--------+---------+
             |                  |
             v                  v
       Web render path      Trust/archive path
       PMTiles + HTTP       hash / signature
       R2/S3 + MapLibre     IPFS / blockchain
             |                  |
             v                  v
           WebGIS          verify / provenance
```

If IPFS or a blockchain registry is unavailable, the reference WebGIS path MUST continue to operate. Storage, gateway and blockchain providers are replaceable; VFM dataset identity is not.

## Core 1.3: what is frozen

VFM 1.3 freezes the **binary Core** before expanding domain profiles. The Core remains intentionally small: container structure, identity, byte addressing, integrity, profile discovery and deterministic conformance.

> **Freeze the core. Extend by profile.**

The protocol model sits **above** this frozen wire layer. Defining VFM as a cross-GIS protocol does not change the already-frozen byte contract.

```text
Layer 4  Applications
         WebGIS / Digital Twin / AI / Robot / Vehicle

Layer 3  Domain Profiles
         Feature / Raster / 3D / BIM / Sensor / Temporal / GeoAI

Layer 2  Cross-GIS Logical Protocol
         identity / frames / time / relations / provenance

Layer 1  VFM Core Binary 1.3
         deterministic container / addressing / integrity / profile discovery
```

## What is real in this repository

This repository includes an executable conformance seed, not only a design document:

- `fixtures/golden/core-minimal.vfm` — a real, deterministic **584-byte VFM 1.3 artifact** containing the required `META`, `HASH` and `PROF` sections.
- `fixtures/golden/core-chunked.vfm` — a real chunked artifact exercising the frozen 128-byte ChunkDescriptor.
- `src/lib.rs` — Rust reference reader/writer for the frozen Core structures.
- `src/bin/vfm.rs` — minimal CLI (`build-minimal`, `validate`, `inspect`).
- `tools/python_reference.py` — an **independent Python generator** that does not call the Rust implementation.
- `tests/conformance.rs` — byte identity + malformed corpus tests.
- `fixtures/malformed/` — fixtures targeting stable VFM error codes.
- `index.html` — VFM 1.3 specification website.
- `spec/v1.3/core-binary.md` — normative text form of the frozen wire contract.
- `spec/v1.3/cross-gis-protocol.md` — conceptual protocol model above the binary Core.

The central Core interoperability proof is reproducible:

```text
VFM 1.3 byte-level specification
          |
    +-----+-----+
    v           v
Rust writer   Python writer
    |           |
    +-----+-----+
          v
  byte-for-byte cmp
          |
          v
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

## Design principles

1. **Protocol first, file second.**
2. **Stable identity over filename identity.**
3. **Interoperability does not require destructive conversion.**
4. **Space and time are explicit, never guessed.**
5. **Relations are first-class data.**
6. **Provenance is part of interoperability.**
7. **Profiles extend the protocol without breaking Core.**
8. **AI is a consumer of VFM, not the definition of VFM.**
9. **Deterministic integrity remains machine-verifiable.**
10. **Existing standards should be bridged, not replaced.**

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

- frozen 256-byte Header and 64-byte Directory Entry;
- frozen 128-byte ChunkDescriptor;
- deterministic byte output for `core-minimal.vfm`;
- Rust reader/writer and independent Python writer byte identity;
- CRC32C + SHA-256 integrity chain;
- stable failures for bad magic, Header CRC, reserved bits, Directory digest, unaligned offsets, overlapping ranges, invalid hash references, content hash mismatch and root digest mismatch;
- GitHub Actions CI that reproduces and compares the binary artifact.

Fast-Open, optional compression and additional domain-profile golden vectors are later conformance expansions; they must not change the frozen Core 1.x structs.

## Website

VFM 1.3 specification site:

**https://base27-cvnss.github.io/VFM/**

The site presents both the cross-GIS protocol positioning and the frozen byte-level Core. GitHub Pages deploys from Actions.

## Current status

- **Repository:** https://github.com/Base27-CVNSS/VFM
- **Specification site:** https://base27-cvnss.github.io/VFM/
- **Protocol positioning:** cross-GIS / cross-dataset spatial data protocol focused on conversion, packaging, integrity, delivery, visualization and preservation.
- **Binary layer:** VFM 1.3 Core Freeze Candidate; the frozen byte contract is unchanged by this architecture update.
- **Reference implementation:** Rust reader/writer + independent Python generator.
- **Conformance:** byte-identical golden fixture + malformed corpus.
- **WebGIS reference stack:** VFM Manifest -> PMTiles/MVT -> static HTTP Range storage -> MapLibre GL JS.
- **Reference storage deployment:** Cloudflare R2 or any equivalent S3/HTTP-compatible object store; VFM does not depend on R2 identity.
- **Optional extensions:** IPFS/content addressing and blockchain registry are outside the render critical path.
- **CI:** Rust stable compiles and runs the conformance suite.
- **Pages:** deployed through GitHub Actions.

The repository commit supplied earlier, `6b80330ff4ff79180ee81d0e792532360b008c0d`, is an earlier Core-fix commit rather than the current repository HEAD. This update advances documentation/architecture only; it does not change the frozen Core 1.3 structs.

VFM is **not** currently an OGC/ISO/IETF standard. Draft 1.0–1.2 were exploratory and are not byte-stable compatibility targets.

## One-sentence positioning

> **VFM is a cross-GIS / cross-dataset data protocol that converts, packages, protects, distributes, visualizes and preserves heterogeneous spatial-temporal data while keeping identity, reference, relationships, provenance and integrity consistent across systems.**

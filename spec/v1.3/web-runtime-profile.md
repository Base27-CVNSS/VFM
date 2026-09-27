# VFM 1.3 — Web Runtime Profile

Status: **Reference profile, non-breaking addition above the frozen VFM 1.3 Core.**

This document defines the recommended minimal WebGIS path for VFM. It does not change any frozen Core 1.3 byte structure.

## 1. Goal

The Web Runtime Profile keeps WebGIS rendering stable and simple while allowing VFM Core to remain focused on dataset identity, structure, addressing and integrity.

The application SHOULD need only one logical entry point:

```text
VFM.open("dataset.vfmm")
```

This is **one logical call**, not one HTTP request. The runtime may perform multiple range/chunk requests for the visible map extent and zoom level.

## 2. Reference stack

```text
dataset.vfmm
    |
    v
VFM Web Profile
    |
    +--> PMTiles / MVT
    |
    v
HTTPS GET + HTTP Range + cache metadata
    |
    v
S3/HTTP-compatible object storage
    |
    v
MapLibre GL JS
    |
    v
WebGIS
```

Reference choices:

- **MapLibre GL JS** — reference renderer;
- **PMTiles/MVT** — recommended vector WebGIS delivery profile;
- **Cloudflare R2** — current reference object-store deployment;
- **S3/HTTP-compatible storage** — portability contract.

None of these provider/product names is part of VFM Dataset Identity.

## 3. Required architectural boundary

VFM Core MUST NOT require:

- a blockchain;
- IPFS;
- an AI/LLM;
- MCP;
- a GIS server;
- a relational/spatial database;
- a specific CDN or cloud vendor.

A baseline VFM WebGIS deployment MUST remain capable of rendering from static HTTP-compatible storage.

## 4. Manifest-first runtime

The manifest is the single logical application entry point and MAY resolve multiple immutable assets.

Example:

```json
{
  "vfm": "1.3",
  "id": "vfm:example:dataset",
  "version": "2026.09.27",
  "core": {
    "href": "./dataset.vfm"
  },
  "web": {
    "renderer": "maplibre",
    "sources": [
      {"type": "pmtiles", "href": "./base.a83f17c.pmtiles"}
    ],
    "style": "./style.json"
  }
}
```

The runtime SHOULD hide storage layout, sharding and provider details from application code.

## 5. Web transport contract

A Web Profile SHOULD be deployable with standard web capabilities:

- HTTPS GET;
- byte-range requests where the profile needs them;
- CORS;
- ETag or equivalent validators;
- Cache-Control;
- immutable/versioned asset URLs.

Large delivery artifacts MAY be sharded. Sharding is an implementation concern hidden by the manifest.

## 6. Canonical data vs delivery representation

VFM canonical data and WebGIS delivery data have different responsibilities:

```text
VFM canonical dataset       Web delivery profile
---------------------       --------------------
full fidelity               optimized for viewport
identity/provenance         LOD/tile-oriented
relations/topology          render-oriented MVT
preservation                fast browser retrieval
```

A PMTiles/MVT representation is therefore a profile/derivative of a VFM dataset, not the definition of VFM itself.

## 7. Optional IPFS and blockchain adapters

IPFS/content addressing and blockchain registries MAY be used for decentralized retrieval, provenance, custody, version history or archival proof.

They are outside the render critical path:

```text
                    VFM
                     |
           +---------+----------+
           |                    |
           v                    v
      WEB RENDER PATH      TRUST/ARCHIVE PATH
      PMTiles + HTTP       hash/signature
      object storage       IPFS/blockchain
      MapLibre             provenance/audit
```

Failure of the trust/archive path MUST NOT make a public baseline WebGIS map unavailable when its HTTP delivery assets are otherwise reachable.

## 8. Reference runtime API

A minimal future runtime can remain small:

```text
open()
manifest()
layers()
getChunk()
queryIndex()
verify()
resolve()
mount()
```

A convenience API MAY expose:

```text
VFM.createMap({
  container: "map",
  dataset: "https://data.example.org/dataset.vfmm"
})
```

The renderer remains replaceable behind this API.

## 9. Core principle

> **One Dataset -> One Manifest -> One Runtime API -> One Data Domain.**

And:

> **Storage is replaceable. Gateway is replaceable. Renderer is replaceable. Blockchain is replaceable. VFM Dataset Identity is not.**

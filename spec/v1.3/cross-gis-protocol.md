# VFM 1.3 — Cross-GIS Protocol Model

Status: **Conceptual interoperability layer for VFM 1.3**. This document defines the intended role of VFM above the frozen binary Core. It does **not** change the byte-level Core 1.3 wire contract in `core-binary.md`.

## 1. Definition

**VFM is a cross-GIS protocol for interoperability between heterogeneous spatial datasets.**

VFM is not primarily another GIS file format, database, map renderer, editing system or AI model. Its purpose is to define a common contract by which independent datasets can identify, reference, relate, verify and exchange information about the same physical or logical world.

A concise statement is:

> **VFM standardizes how spatial datasets understand each other, not how every dataset must be physically stored.**

The `.vfm` binary artifact is one normative transport/container encoding of this protocol. It is not the protocol itself.

```text
VFM Protocol
    |
    +-- Logical interoperability model
    |     identity
    |     spatial reference / frame
    |     time
    |     relations / topology
    |     state
    |     semantics bindings
    |     provenance
    |     integrity
    |
    +-- Profiles
    |     Feature / Raster / Point Cloud / 3D / BIM
    |     Temporal / Sensor / GeoAI / Digital Twin / Robotics
    |
    +-- Encodings / transports
          .vfm binary
          API / stream
          database representation
          future canonical serializations
```

Therefore:

```text
VFM != .vfm
.vfm = one encoding of the VFM Protocol
```

## 2. The root problem

The root problem in spatial digital transformation is not merely that data exists in many formats. The deeper problem is that different systems may describe the **same real-world entity** without a stable, machine-verifiable way to know that they refer to the same thing, in the same space and time.

A bridge, road, parcel or building may simultaneously exist as:

- a GIS feature;
- a GeoParquet/GeoPackage record;
- a raster/orthophoto observation;
- a LiDAR segment;
- an IFC/BIM object;
- a 3D Tiles object;
- a sensor asset;
- a SLAM landmark;
- a trajectory constraint;
- a Digital Twin object.

VFM does not require these sources to be converted into one physical representation. Instead, it provides a protocol layer that can bind them to common identity, reference frames, time, relationships and provenance.

```text
Dataset A ----\
Dataset B -----\
Dataset C ------> VFM interoperability contract ---> Applications
Dataset D -----/                                 GIS / BIM / AI
Dataset E ----/                                  Digital Twin / Robot
```

## 3. Core interoperability primitives

A VFM-compatible profile SHOULD make the following concepts explicit when applicable.

### 3.1 Identity

A logical or physical entity needs a stable identity independent of storage-specific IDs.

Example:

```text
VFM entity: bridge:001
  maps_to GIS OBJECTID  = 117
  maps_to OSM ID        = 4839201
  maps_to IFC GlobalId  = 2X8...
  maps_to LiDAR segment = seg-184
  maps_to sensor asset  = strain-27
```

VFM identity does not replace native IDs. It links them.

### 3.2 Spatial reference and coordinate frames

A dataset must be able to state, when relevant:

- horizontal CRS;
- vertical datum/reference;
- axis order;
- unit;
- local or world frame;
- transform between frames;
- temporal validity of that transform.

This is essential for GIS, BIM, LiDAR, SLAM, drones, vehicles and robots because individually valid datasets can become incorrect when combined in incompatible reference frames.

### 3.3 Time and state

VFM distinguishes an entity from its changing observations or state.

```text
entity = building:001
geometry(t)
state(t)
sensor(t)
condition(t)
observation(t)
```

This permits evolution from static GIS toward time-aware Digital Twin and physical-world models.

### 3.4 Relations and topology

VFM profiles may expose explicit relations such as:

```text
parcel:12 contains building:7
bridge:1 crosses river:4
sensor:27 attached_to bridge:1
road:A connected_to road:B
vehicle:3 on road:A
obstacle:9 blocks lane:2
```

The protocol should distinguish observed relations, authoritative relations and derived/computed relations.

### 3.5 Semantics

Semantics should be bindable without forcing every dataset to rewrite its native schema. Profiles may map native fields/classes to controlled vocabularies, ontologies or external semantic dictionaries.

### 3.6 Provenance

Interoperability without provenance is unsafe. A conforming data exchange should be able to describe:

- source/provider;
- acquisition or retrieval time;
- transformation history;
- software/process version;
- license/restrictions;
- validation status;
- content hashes where applicable.

### 3.7 Integrity and deterministic identity

The frozen Core 1.3 binary contract provides deterministic byte-level structures, CRC32C and SHA-256 based integrity. Logical profiles build on that mechanism without redefining the frozen Core.

## 4. Preserve source-native data

A central VFM rule is:

> **Interoperability does not require destructive conversion.**

VFM may reference or encapsulate modality-specific data, but should not require a LiDAR point cloud to become a vector layer, an IFC model to become GeoJSON, or a raster to become features merely to participate in the protocol.

```text
road:001
  geometry  -> roads.parquet / feature 837
  lidar     -> scan.laz / chunk 184
  bim       -> road.ifc / GlobalId ...
  imagery   -> ortho_2026.tif / window ...
  sensor    -> Sensor API / asset ...
```

The protocol binds the representations; it does not erase their native strengths.

## 5. VFM in digital transformation

A conventional digitization pipeline often produces isolated digital files:

```text
paper/CAD/survey/sensor
        |
        v
many independent digital datasets
```

A VFM-oriented pipeline targets interoperable digital knowledge:

```text
Digitization
    |
    v
QA/QC + native standards
    |
    v
VFM identity / space / time / relation / provenance
    |
    v
interoperable spatial data ecosystem
    |
    +--> WebGIS
    +--> BIM
    +--> analytics
    +--> Digital Twin
    +--> GeoAI
    +--> autonomous systems
```

This is why VFM addresses a root problem of spatial digital transformation: **maintaining a consistent digital identity of the real world across datasets and systems**.

## 6. Relationship to existing standards and formats

VFM should bridge mature standards rather than replace them.

Examples of source/profile ecosystems include:

- GeoJSON, GeoPackage and GeoParquet for vector data;
- COG and other raster formats;
- LAS/LAZ for point clouds;
- IFC for BIM;
- CityGML and 3D Tiles for semantic/streaming 3D;
- sensor and IoT APIs;
- SLAM/trajectory representations;
- databases and web services.

The VFM contribution is the cross-dataset contract above these representations.

```text
GeoJSON -----\
GeoParquet ---\
COG -----------\
LAS/LAZ --------> VFM Protocol ---> consumer systems
IFC ------------/                  GIS / BIM / Twin
3D Tiles ------/                   AI / Robot / Vehicle
Sensors ------/
```

## 7. Relationship to AI and world models

VFM is not an AI model and does not require AI.

AI is one consumer of VFM-normalized relationships. A spatial foundation model can derive training views from VFM without treating VFM itself as a training-file format.

```text
VFM Protocol
    |
    v
aligned entities + topology + time + observations + provenance
    |
    +--> text/semantic view
    +--> graph/topology view
    +--> raster view
    +--> point-cloud view
    +--> BIM/3D view
    +--> trajectory/state view
    |
    v
Spatial AI / GeoAI / World Model
```

A world model may use VFM to represent state consistently, but VFM itself remains the interoperability protocol.

## 8. Layering rule for VFM 1.x

The architecture is intentionally layered:

```text
Layer 4  Applications
         WebGIS / Digital Twin / AI / Robot / Vehicle

Layer 3  Domain Profiles
         Feature / Raster / 3D / BIM / Sensor / Temporal / GeoAI

Layer 2  VFM Logical Interoperability Protocol
         identity / references / relations / time / provenance

Layer 1  VFM Core Binary
         deterministic container / addressing / integrity / profile discovery
```

**Core Freeze Candidate 1.3 freezes Layer 1.**  
Layers 2 and 3 may evolve by additive specification and profiles as long as they do not change frozen Core 1.x wire structures.

## 9. Design principles

VFM development should follow these principles:

1. **Protocol first, file second.**
2. **Stable identity over filename identity.**
3. **Reference, do not destructively convert, when the source format is already appropriate.**
4. **Space and time are explicit, never guessed.**
5. **Relations are first-class data.**
6. **Provenance is part of interoperability.**
7. **Profiles extend the protocol without breaking Core.**
8. **Deterministic computation and integrity remain machine-verifiable.**
9. **AI is a consumer, not the definition of VFM.**
10. **The protocol should let old and new systems coexist.**

## 10. One-sentence positioning

> **VFM is a cross-GIS protocol that maintains consistent identity, spatial/temporal reference, relationships, provenance and integrity across heterogeneous representations of the same world.**

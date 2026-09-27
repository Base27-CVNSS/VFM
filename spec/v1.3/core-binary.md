# VFM 1.3 Core Binary Specification — Frozen Wire Contract

Status: **Core Freeze Candidate**. This document is normative for the reference implementation in this repository. Domain profiles are outside this Core.

## 1. Primitive rules

- Byte order: **little-endian only** for all multi-byte integer fields.
- FourCC: four ASCII bytes in display order; never endian-swapped.
- All offsets/lengths: unsigned 64-bit.
- Every `offset + length` operation must be overflow-checked before access.
- Non-zero section offsets, descriptor-table offsets and chunk payload offsets are multiples of 8.
- Padding is zero and is not included in payload length, CRC or content hash.
- Reserved Core 1.x bits/bytes are zero.

Magic bytes at offset 0:

```text
56 46 4D 00 0D 0A 1A 0A
V  F  M \0 \r \n \x1A \n
```

## 2. Fixed Header — 256 bytes

| Off | Size | Type | Field | Contract |
|---:|---:|---|---|---|
|0|8|bytes|magic|fixed magic|
|8|2|u16|format_major|1|
|10|2|u16|format_minor|3|
|12|2|u16|header_size|256|
|14|2|u16|directory_entry_size|64|
|16|1|u8|byte_order|1 = little-endian|
|17|1|u8|header_revision|1|
|18|1|u8|core_hash_algorithm|1 = SHA-256|
|19|1|u8|meta_codec|1 = deterministic CBOR|
|20|4|u32|header_flags|defined below|
|24|4|u32|section_count|0..252; useful Core has META/HASH/PROF|
|28|4|u32|reserved0|0|
|32|8|u64|directory_offset|256|
|40|8|u64|directory_length|`section_count * 64`|
|48|8|u64|manifest_offset|META offset|
|56|8|u64|manifest_length|META stored/raw length|
|64|8|u64|bootstrap_end|0 unless Fast-Open is declared|
|72|8|u64|footer_offset|0 in Core 1.3|
|80|8|u64|declared_file_size|artifact byte length|
|88|16|bytes|dataset_uuid|non-zero 16-byte UUID identity|
|104|8|u64|generation|0 in immutable Core 1.3|
|112|32|bytes|content_root_digest|SHA-256 decoded HASH payload|
|144|32|bytes|directory_digest|SHA-256 exact Directory bytes|
|176|32|bytes|manifest_digest|SHA-256 decoded canonical META bytes|
|208|16|bytes|build_id|all zero for deterministic build|
|224|8|u64|artifact_created_unix_ms|0 for deterministic build|
|232|8|u64|reserved_time|0|
|240|12|bytes|reserved1|zero|
|252|4|u32|header_crc32c|CRC32C bytes 0..251|

### Header flag bits

- bit0 `FASTOPEN_CONFORMANT`
- bit1 `HASH_TABLE_PRESENT`
- bit2 `PROFILE_TABLE_PRESENT`
- bit3 `SIGNATURE_PRESENT`
- bit4 `ENCRYPTED_CONTENT_PRESENT`
- bit5 `DETERMINISTIC_BUILD`
- bit6 `STRICT_INTEGRITY`
- bit7 `IMMUTABLE_RELEASE`
- bits8..31 reserved = 0

## 3. Root Section Directory — 64 bytes per entry

The Directory starts at byte 256 and ends no later than byte 16384.

| Off | Size | Type | Field |
|---:|---:|---|---|
|0|4|bytes|type FourCC|
|4|4|u32|flags|
|8|8|u64|logical_id|
|16|8|u64|offset|
|24|8|u64|stored_length|
|32|8|u64|raw_length|
|40|2|u16|profile_id|
|42|2|u16|section_version|
|44|2|u16|compression_id|
|46|2|u16|encryption_id|
|48|4|u32|crc32c|
|52|4|u32|hash_ref; `0xffffffff` = none|
|56|8|u64|aux|

Section flag bits: `CRITICAL`, `CONTENT_SIGNIFICANT`, `HOT`, `CHUNKED`, `INDEX`, `COMPRESSED`, `ENCRYPTED`, `IMMUTABLE`, `PROFILE_PRIVATE`, `ORDERED` at bits 0..9. Bits 10..31 are zero.

Required Core sections:

- `META`, logical_id=1
- `HASH`, logical_id=2
- `PROF`, logical_id=3

## 4. Chunk Descriptor — 128 bytes

| Off | Size | Type | Field |
|---:|---:|---|---|
|0|8|u64|chunk_id|
|8|8|u64|parent_section_id|
|16|8|u64|logical_key_hi|
|24|8|u64|logical_key_lo|
|32|8|u64|offset|
|40|8|u64|stored_length|
|48|8|u64|raw_length|
|56|4|u32|flags|
|60|4|u32|crc32c|
|64|2|u16|compression_id|
|66|2|u16|encryption_id|
|68|4|u32|hash_ref|
|72|8|u64|aux0|
|80|8|u64|aux1|
|88|8|u64|aux2|
|96|32|bytes|reserved = zero|

Chunk flags at bits 0..4: `CONTENT_SIGNIFICANT`, `COMPRESSED`, `ENCRYPTED`, `ORDERED`, `SPARSE`; remaining bits are zero.

## 5. Codec IDs

Compression `u16`:

- `0x0000 NONE` — mandatory
- `0x0001 ZSTD`
- `0x0002 GZIP`
- `0x0003 BROTLI`
- `0x0004 raw DEFLATE`
- `0x0005..0x7fff` public registry reserved
- `0x8000..0xfffe` private/experimental
- `0xffff INVALID`

Encryption `u16`:

- `0x0000 NONE` — Core mandatory
- `0x0001..0x7fff` security-profile registry
- `0x8000..0xfffe` private/experimental
- `0xffff INVALID`

Hash algorithm `u8`: `1 = SHA-256`. META codec `u8`: `1 = deterministic CBOR`.

## 6. META

META is deterministic CBOR, no indefinite-length items. Required integer keys:

- `0`: `[1,3]`
- `1`: `bstr(16)` equal to Header dataset_uuid
- `4`: profile table logical ID = 3
- `5`: hash table logical ID = 2

Key 6 is required when `FASTOPEN_CONFORMANT` is set. Core-reserved keys 9..1023 are not assigned by 1.3.

## 7. HashRecord — 48 bytes

| Off | Size | Type | Field |
|---:|---:|---|---|
|0|1|u8|object_kind: 1=SECTION, 2=CHUNK|
|1|1|u8|hash_algorithm = 1|
|2|1|u8|digest_length = 32|
|3|1|u8|flags: bit0 ROOT_INCLUDED; bit1 RAW_LOGICAL|
|4|8|u64|object_id|
|12|32|bytes|SHA-256 digest of raw logical bytes|
|44|4|u32|reserved = 0|

HASH payload is a strict increasing sequence by `(object_kind, object_id)`, with no duplicates.

```text
content_root_digest = SHA256(exact decoded HASH payload bytes)
directory_digest    = SHA256(exact Directory bytes)
manifest_digest     = SHA256(decoded canonical META bytes)
header_crc32c       = CRC32C(Header[0:252])
```

Compression level, physical offset and padding do not change logical content identity because object digests are computed after decrypt/decompress on raw logical bytes.

## 8. Failure codes used by the conformance seed

- `E0001 BAD_MAGIC`
- `E0005 HEADER_CRC_MISMATCH`
- `E0006 RESERVED_NONZERO`
- `E0012 DIRECTORY_DIGEST_MISMATCH`
- `E0014 OVERLAPPING_RANGES`
- `E0015 UNALIGNED_OFFSET`
- `E0032 HASH_REF_OUT_OF_RANGE`
- `E0033 CONTENT_HASH_MISMATCH`
- `E0034 ROOT_DIGEST_MISMATCH`

## 9. Golden artifact

`fixtures/golden/core-minimal.vfm`:

- file size: 584 bytes
- dataset UUID bytes: `00112233445566778899aabbccddeeff`
- META offset: 448, length: 27
- PROF offset: 480, length: 1
- HASH offset: 488, length: 96
- artifact SHA-256: `5b65b13e2ae03aa3697dc869edf5168e6601be7465b1eba10331c145ebaaeefc`
- content root: `63978b1b7d47f8be14cf981d6eb0329c0378ff3dd186c1b51cc0e991fa695340`

The Rust writer and the independent Python writer must produce this file byte-for-byte.

#!/usr/bin/env python3
"""Independent Python reference generator for the VFM 1.3 Core Freeze Candidate.

This file intentionally does not call the Rust implementation. It is used by CI
as a second implementation to prove that the byte-level specification is
sufficient to reproduce the same golden artifact.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
from pathlib import Path

MAGIC = bytes.fromhex("56 46 4D 00 0D 0A 1A 0A")
HEADER_SIZE = 256
DIR_ENTRY_SIZE = 64
HASH_RECORD_SIZE = 48
NONE_HASH_REF = 0xFFFFFFFF
DATASET_UUID = bytes.fromhex("00112233445566778899aabbccddeeff")

HASH_TABLE_PRESENT = 1 << 1
PROFILE_TABLE_PRESENT = 1 << 2
DETERMINISTIC_BUILD = 1 << 5
STRICT_INTEGRITY = 1 << 6
IMMUTABLE_RELEASE = 1 << 7
HEADER_FLAGS = HASH_TABLE_PRESENT | PROFILE_TABLE_PRESENT | DETERMINISTIC_BUILD | STRICT_INTEGRITY | IMMUTABLE_RELEASE

CRITICAL = 1 << 0
CONTENT_SIGNIFICANT = 1 << 1
HOT = 1 << 2
IMMUTABLE = 1 << 7

def align(value: int, alignment: int = 8) -> int:
    return (value + alignment - 1) & ~(alignment - 1)

def crc32c(data: bytes) -> int:
    crc = 0xFFFFFFFF
    poly = 0x82F63B78
    for b in data:
        crc ^= b
        for _ in range(8):
            crc = (crc >> 1) ^ (poly if (crc & 1) else 0)
    return crc ^ 0xFFFFFFFF

assert crc32c(b"123456789") == 0xE3069283

def sha256(data: bytes) -> bytes:
    return hashlib.sha256(data).digest()

def cbor_uint(n: int) -> bytes:
    if n < 24: return bytes([n])
    if n <= 0xFF: return b"\x18" + bytes([n])
    if n <= 0xFFFF: return b"\x19" + struct.pack(">H", n)
    if n <= 0xFFFFFFFF: return b"\x1a" + struct.pack(">I", n)
    return b"\x1b" + struct.pack(">Q", n)

def cbor_bstr(value: bytes) -> bytes:
    if len(value) < 24: return bytes([0x40 | len(value)]) + value
    raise ValueError("fixture helper only supports short bstr")

def cbor_tstr(value: str) -> bytes:
    raw = value.encode("utf-8")
    if len(raw) < 24: return bytes([0x60 | len(raw)]) + raw
    raise ValueError("fixture helper only supports short tstr")

def cbor_array(items: list[bytes]) -> bytes:
    if len(items) >= 24: raise ValueError("fixture helper only supports short arrays")
    return bytes([0x80 | len(items)]) + b"".join(items)

def cbor_map_int(items: list[tuple[int, bytes]]) -> bytes:
    items = sorted(items, key=lambda kv: kv[0])
    if len(items) >= 24: raise ValueError("fixture helper only supports short maps")
    return bytes([0xA0 | len(items)]) + b"".join(cbor_uint(k) + v for k, v in items)

def minimal_meta(dataset_uuid: bytes = DATASET_UUID) -> bytes:
    return cbor_map_int([
        (0, cbor_array([cbor_uint(1), cbor_uint(3)])),
        (1, cbor_bstr(dataset_uuid)),
        (4, cbor_uint(3)),
        (5, cbor_uint(2)),
    ])

def minimal_prof() -> bytes:
    return cbor_array([])

def hash_record(object_kind: int, object_id: int, digest: bytes) -> bytes:
    assert len(digest) == 32
    return struct.pack("<BBBBQ", object_kind, 1, 32, 0x03, object_id) + digest + struct.pack("<I", 0)

def dir_entry(fourcc: bytes, flags: int, logical_id: int, offset: int, stored_length: int,
              raw_length: int, profile_id: int, section_version: int, compression_id: int,
              encryption_id: int, crc: int, hash_ref: int, aux: int = 0) -> bytes:
    assert len(fourcc) == 4
    out = bytearray(64)
    out[0:4] = fourcc
    struct.pack_into("<IQQQQHHHHIIQ", out, 4, flags, logical_id, offset, stored_length,
                     raw_length, profile_id, section_version, compression_id, encryption_id,
                     crc, hash_ref, aux)
    return bytes(out)

def build_core_minimal() -> tuple[bytes, dict]:
    meta = minimal_meta()
    prof = minimal_prof()
    meta_hash = sha256(meta)
    prof_hash = sha256(prof)
    hash_payload = hash_record(1, 1, meta_hash) + hash_record(1, 3, prof_hash)
    content_root = sha256(hash_payload)

    section_count = 3
    directory_offset = HEADER_SIZE
    directory_length = section_count * DIR_ENTRY_SIZE
    meta_offset = align(directory_offset + directory_length, 64)
    prof_offset = align(meta_offset + len(meta), 8)
    hash_offset = align(prof_offset + len(prof), 8)
    file_size = hash_offset + len(hash_payload)

    meta_entry = dir_entry(b"META", CRITICAL | CONTENT_SIGNIFICANT | HOT | IMMUTABLE,
        1, meta_offset, len(meta), len(meta), 0, 1, 0, 0, crc32c(meta), 0)
    hash_entry = dir_entry(b"HASH", CRITICAL | IMMUTABLE,
        2, hash_offset, len(hash_payload), len(hash_payload), 0, 1, 0, 0,
        crc32c(hash_payload), NONE_HASH_REF)
    prof_entry = dir_entry(b"PROF", CRITICAL | CONTENT_SIGNIFICANT | HOT | IMMUTABLE,
        3, prof_offset, len(prof), len(prof), 0, 1, 0, 0, crc32c(prof), 1)
    directory = meta_entry + hash_entry + prof_entry

    header = bytearray(HEADER_SIZE)
    header[0:8] = MAGIC
    struct.pack_into("<HHHHBBBB", header, 8, 1, 3, 256, 64, 1, 1, 1, 1)
    struct.pack_into("<III", header, 20, HEADER_FLAGS, section_count, 0)
    struct.pack_into("<QQQQQQQ", header, 32, directory_offset, directory_length,
                     meta_offset, len(meta), 0, 0, file_size)
    header[88:104] = DATASET_UUID
    struct.pack_into("<Q", header, 104, 0)
    header[112:144] = content_root
    header[144:176] = sha256(directory)
    header[176:208] = sha256(meta)
    struct.pack_into("<QQ", header, 224, 0, 0)
    struct.pack_into("<I", header, 252, crc32c(bytes(header[:252])))

    out = bytearray(file_size)
    out[0:256] = header
    out[directory_offset:directory_offset + len(directory)] = directory
    out[meta_offset:meta_offset + len(meta)] = meta
    out[prof_offset:prof_offset + len(prof)] = prof
    out[hash_offset:hash_offset + len(hash_payload)] = hash_payload

    info = {
        "artifact_sha256": hashlib.sha256(out).hexdigest(),
        "file_size": file_size,
        "dataset_uuid": DATASET_UUID.hex(),
        "content_root_digest": content_root.hex(),
        "directory_digest": sha256(directory).hex(),
        "manifest_digest": sha256(meta).hex(),
        "header_crc32c": f"{struct.unpack_from('<I', header, 252)[0]:08x}",
    }
    return bytes(out), info

def build_core_chunked() -> tuple[bytes, dict]:
    meta = minimal_meta()
    prof = cbor_array([cbor_tstr("org.vietflex.fixture/1")])
    chunk_payload = b"VFM-CHUNK-001"
    meta_hash, prof_hash, chunk_hash = sha256(meta), sha256(prof), sha256(chunk_payload)
    hash_payload = hash_record(1, 1, meta_hash) + hash_record(1, 3, prof_hash) + hash_record(2, 1001, chunk_hash)
    content_root = sha256(hash_payload)

    section_count = 4
    directory_offset = HEADER_SIZE
    directory_length = section_count * DIR_ENTRY_SIZE
    meta_offset = align(directory_offset + directory_length, 64)
    prof_offset = align(meta_offset + len(meta), 8)
    desc_offset = align(prof_offset + len(prof), 8)
    chunk_offset = align(desc_offset + 128, 8)
    hash_offset = align(chunk_offset + len(chunk_payload), 8)
    file_size = hash_offset + len(hash_payload)

    desc = bytearray(128)
    struct.pack_into("<QQQQQQQIIHHIQQQ", desc, 0, 1001, 256, 0, 1, chunk_offset,
                     len(chunk_payload), len(chunk_payload), 1, crc32c(chunk_payload),
                     0, 0, 2, 0, 0, 0)
    desc = bytes(desc)

    meta_entry = dir_entry(b"META", CRITICAL | CONTENT_SIGNIFICANT | HOT | IMMUTABLE,
        1, meta_offset, len(meta), len(meta), 0, 1, 0, 0, crc32c(meta), 0)
    hash_entry = dir_entry(b"HASH", CRITICAL | IMMUTABLE,
        2, hash_offset, len(hash_payload), len(hash_payload), 0, 1, 0, 0, crc32c(hash_payload), NONE_HASH_REF)
    prof_entry = dir_entry(b"PROF", CRITICAL | CONTENT_SIGNIFICANT | HOT | IMMUTABLE,
        3, prof_offset, len(prof), len(prof), 0, 1, 0, 0, crc32c(prof), 1)
    extn_entry = dir_entry(b"EXTN", (1 << 3) | IMMUTABLE | (1 << 8),
        256, desc_offset, len(desc), len(chunk_payload), 1, 1, 0, 0, crc32c(desc), NONE_HASH_REF, 1)
    directory = meta_entry + hash_entry + prof_entry + extn_entry

    header = bytearray(HEADER_SIZE)
    header[0:8] = MAGIC
    struct.pack_into("<HHHHBBBB", header, 8, 1, 3, 256, 64, 1, 1, 1, 1)
    struct.pack_into("<III", header, 20, HEADER_FLAGS, section_count, 0)
    struct.pack_into("<QQQQQQQ", header, 32, directory_offset, directory_length,
                     meta_offset, len(meta), 0, 0, file_size)
    header[88:104] = DATASET_UUID
    struct.pack_into("<Q", header, 104, 0)
    header[112:144] = content_root
    header[144:176] = sha256(directory)
    header[176:208] = sha256(meta)
    struct.pack_into("<QQ", header, 224, 0, 0)
    struct.pack_into("<I", header, 252, crc32c(bytes(header[:252])))

    out = bytearray(file_size)
    out[:256] = header
    out[directory_offset:directory_offset + len(directory)] = directory
    out[meta_offset:meta_offset + len(meta)] = meta
    out[prof_offset:prof_offset + len(prof)] = prof
    out[desc_offset:desc_offset + len(desc)] = desc
    out[chunk_offset:chunk_offset + len(chunk_payload)] = chunk_payload
    out[hash_offset:hash_offset + len(hash_payload)] = hash_payload
    return bytes(out), {"artifact_sha256": hashlib.sha256(out).hexdigest(), "file_size": file_size}

def recalc_header_crc(buf: bytearray) -> None:
    struct.pack_into("<I", buf, 252, crc32c(bytes(buf[:252])))

def recalc_directory_digest_and_header_crc(buf: bytearray) -> None:
    count = struct.unpack_from("<I", buf, 24)[0]
    off = struct.unpack_from("<Q", buf, 32)[0]
    length = count * 64
    buf[144:176] = sha256(bytes(buf[off:off + length]))
    recalc_header_crc(buf)

def malformed_variants(valid: bytes) -> dict[str, tuple[bytes, str]]:
    variants = {}
    b=bytearray(valid); b[0]^=1; variants["bad-magic.vfm"]=(bytes(b),"E0001")
    b=bytearray(valid); b[255]^=1; variants["bad-header-crc.vfm"]=(bytes(b),"E0005")
    b=bytearray(valid); struct.pack_into("<I",b,28,1); recalc_header_crc(b); variants["reserved-nonzero.vfm"]=(bytes(b),"E0006")
    b=bytearray(valid); b[260]^=0x80; variants["bad-directory-digest.vfm"]=(bytes(b),"E0012")
    b=bytearray(valid); struct.pack_into("<Q",b,272,449); recalc_directory_digest_and_header_crc(b); variants["unaligned-offset.vfm"]=(bytes(b),"E0015")
    b=bytearray(valid); struct.pack_into("<Q",b,256+2*64+16,448); recalc_directory_digest_and_header_crc(b); variants["overlap.vfm"]=(bytes(b),"E0014")
    b=bytearray(valid); struct.pack_into("<I",b,256+52,99); recalc_directory_digest_and_header_crc(b); variants["hash-ref-out-of-range.vfm"]=(bytes(b),"E0032")
    b=bytearray(valid)
    mo,ml=struct.unpack_from("<Q",b,48)[0],struct.unpack_from("<Q",b,56)[0]
    b[mo+7]^=1
    nm=bytes(b[mo:mo+ml])
    struct.pack_into("<I",b,256+48,crc32c(nm)); b[176:208]=sha256(nm); recalc_directory_digest_and_header_crc(b)
    variants["bad-content-hash.vfm"]=(bytes(b),"E0033")
    b=bytearray(valid); b[112]^=1; recalc_header_crc(b); variants["bad-root-digest.vfm"]=(bytes(b),"E0034")
    return variants

def write_fixture_tree(root: Path) -> dict:
    gd=root/"fixtures"/"golden"; md=root/"fixtures"/"malformed"; gd.mkdir(parents=True,exist_ok=True); md.mkdir(parents=True,exist_ok=True)
    valid,info=build_core_minimal(); (gd/"core-minimal.vfm").write_bytes(valid)
    chunked,cinfo=build_core_chunked(); (gd/"core-chunked.vfm").write_bytes(chunked)
    manifest={}
    for name,(data,code) in malformed_variants(valid).items():
        (md/name).write_bytes(data); manifest[name]={"expected_error":code,"sha256":hashlib.sha256(data).hexdigest(),"size":len(data)}
    (md/"manifest.json").write_text(json.dumps(manifest,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    return {"golden":info,"chunked":cinfo,"malformed":manifest}

def main() -> None:
    parser=argparse.ArgumentParser()
    parser.add_argument("--output",type=Path)
    parser.add_argument("--fixtures-root",type=Path)
    args=parser.parse_args()
    data,info=build_core_minimal()
    if args.output:
        args.output.parent.mkdir(parents=True,exist_ok=True); args.output.write_bytes(data)
    if args.fixtures_root: write_fixture_tree(args.fixtures_root)
    if not args.output and not args.fixtures_root: print(json.dumps(info,indent=2))

if __name__ == "__main__":
    main()

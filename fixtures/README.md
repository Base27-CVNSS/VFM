# VFM 1.3 conformance fixtures

## Golden

`golden/core-minimal.vfm` is the canonical minimum artifact for the frozen Core. It contains only the three required Core sections: `META`, `HASH`, `PROF`.

`golden/core-chunked.vfm` additionally exercises one 128-byte ChunkDescriptor and a content-significant chunk hash.

The minimal artifact is deterministic. Both implementations must produce SHA-256:

`5b65b13e2ae03aa3697dc869edf5168e6601be7465b1eba10331c145ebaaeefc`

## Malformed corpus

The malformed files are derived from the golden artifact but carefully recompute earlier validation layers when required so the reader reaches the intended stable error code. See `malformed/manifest.json`.

These fixtures are intentionally small; do not treat them as GIS feature-profile examples. They exercise the Core container contract only.

use std::{fs, path::PathBuf};

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn rust_writer_is_byte_identical_to_golden() {
    let expected = fs::read(repo_path("fixtures/golden/core-minimal.vfm")).unwrap();
    let actual = vfm_core::write_core_minimal();
    assert_eq!(actual, expected);
}

#[test]
fn golden_is_valid() {
    let data = fs::read(repo_path("fixtures/golden/core-minimal.vfm")).unwrap();
    let v = vfm_core::read(&data).unwrap();
    assert_eq!(v.header.format_major, 1);
    assert_eq!(v.header.format_minor, 3);
    assert_eq!(v.directory.len(), 3);
    assert!(v.profiles.is_empty());
    assert!(v.warnings.is_empty());
    assert_eq!(vfm_core::artifact_sha256(&data), "5b65b13e2ae03aa3697dc869edf5168e6601be7465b1eba10331c145ebaaeefc");
}

#[test]
fn deterministic_writer_is_stable() {
    assert_eq!(vfm_core::write_core_minimal(), vfm_core::write_core_minimal());
}

#[test]
fn chunked_golden_validates_frozen_128_byte_descriptor() {
    let data = fs::read(repo_path("fixtures/golden/core-chunked.vfm")).unwrap();
    let v = vfm_core::read(&data).unwrap();
    assert_eq!(v.directory.len(), 4);
    assert_eq!(v.profiles, vec!["org.vietflex.fixture/1".to_string()]);
    assert_eq!(vfm_core::artifact_sha256(&data), "252bffa51b99534ccbdce3a8c07e9044090a783e773dda031f51a5cffda4ef9c");
}

#[test]
fn malformed_corpus_returns_expected_codes() {
    let cases = [
        ("bad-magic.vfm", vfm_core::ErrorCode::E0001),
        ("bad-header-crc.vfm", vfm_core::ErrorCode::E0005),
        ("reserved-nonzero.vfm", vfm_core::ErrorCode::E0006),
        ("bad-directory-digest.vfm", vfm_core::ErrorCode::E0012),
        ("unaligned-offset.vfm", vfm_core::ErrorCode::E0015),
        ("overlap.vfm", vfm_core::ErrorCode::E0014),
        ("hash-ref-out-of-range.vfm", vfm_core::ErrorCode::E0032),
        ("bad-content-hash.vfm", vfm_core::ErrorCode::E0033),
        ("bad-root-digest.vfm", vfm_core::ErrorCode::E0034),
    ];
    for (name, code) in cases {
        let data = fs::read(repo_path(&format!("fixtures/malformed/{name}"))).unwrap();
        let err = vfm_core::read(&data).unwrap_err();
        assert_eq!(err.code, code, "fixture {name}: {err}");
    }
}

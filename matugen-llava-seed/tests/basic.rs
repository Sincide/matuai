use assert_cmd::Command;
use matugen_llava_seed::utils::{validate_hex, decide_mode};

#[test]
fn test_hex_validator() {
    assert!(validate_hex("#A1B2C3"));
    assert!(!validate_hex("123456"));
}

#[test]
fn test_decide_mode() {
    assert_eq!(decide_mode(0.3), "dark");
    assert_eq!(decide_mode(0.7), "light");
}

#[test]
fn test_cli_analyze_missing_file() {
    let mut cmd = Command::cargo_bin("matugen-llava-seed").unwrap();
    cmd.arg("analyze").arg("--image").arg("/no/such/file.jpg");
    cmd.assert().failure();
}

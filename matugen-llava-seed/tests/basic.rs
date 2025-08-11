use assert_cmd::Command;
use matugen_llava_seed::utils::{validate_hex, decide_mode};
use matugen_llava_seed::llava::parse_palette;

#[test]
fn test_hex_validator() {
    assert!(validate_hex("#A1B2C3"));
    assert!(!validate_hex("123456"));
}

#[test]
fn test_parse_palette() {
    let json = "{\"primary_hex\":\"#AABBCC\",\"secondary_hex\":\"#112233\",\"tertiary_hex\":\"#445566\",\"accent1_hex\":\"#778899\",\"accent2_hex\":\"#ABCDEF\",\"neutral1_hex\":\"#123456\",\"neutral2_hex\":\"#654321\"}";
    let palette = parse_palette(json).expect("palette");
    assert_eq!(palette.len(), 7);
    for color in palette.values() {
        assert!(validate_hex(color));
    }
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

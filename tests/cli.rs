use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn recommends_a_game_when_password_source_is_missing() {
    cargo_bin_cmd!("monke")
        .args(["unpack", "input", "--output", "output"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "provide a game with --game (preferred), or a password with --password",
        ))
        .stderr(predicate::str::contains(
            "Usage: monke unpack [OPTIONS] --output <OUTPUT> <INPUT>",
        ))
        .stderr(predicate::str::contains(
            "For more information, try '--help'.",
        ));
}

#[test]
fn pack_inspect_and_unpack_round_trip() {
    let directory = tempdir().unwrap();
    let json_path = directory.path().join("Profile.Save.json");
    let save_path = directory.path().join("Profile.Save");
    let unpacked_path = directory.path().join("unpacked.json");
    let contents = b"\xef\xbb\xbf{\"name\":\"monke\"}";
    fs::write(&json_path, contents).unwrap();

    cargo_bin_cmd!("monke")
        .args(["pack"])
        .arg(&json_path)
        .args(["--output"])
        .arg(&save_path)
        .args(["--game", "btd6"])
        .assert()
        .success();

    cargo_bin_cmd!("monke")
        .args(["inspect"])
        .arg(&save_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("header version: 2\n"))
        .stdout(predicate::str::contains("header length: 37\n"));

    cargo_bin_cmd!("monke")
        .args(["unpack"])
        .arg(&save_path)
        .args(["--output"])
        .arg(&unpacked_path)
        .args(["--password", "11"])
        .assert()
        .success();

    assert_eq!(fs::read(unpacked_path).unwrap(), contents);
}

#[test]
fn refuses_to_overwrite_without_force() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("input.json");
    let output = directory.path().join("Profile.Save");
    fs::write(&input, b"{}").unwrap();
    fs::write(&output, b"keep me").unwrap();

    cargo_bin_cmd!("monke")
        .args(["pack"])
        .arg(&input)
        .args(["--output"])
        .arg(&output)
        .args(["--password", "11"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    assert_eq!(fs::read(output).unwrap(), b"keep me");
}

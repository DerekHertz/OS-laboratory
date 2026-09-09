use std::process::Command;

#[test]
fn real_native_process_accepts_values_and_rejects_invalid_input() {
    for (args, expected) in [
        (vec!["17", "25"], Some("42")),
        (vec!["4294967295", "1"], Some("0")),
        (vec!["-1", "2"], None),
        (vec!["4294967296", "0"], None),
        (vec![], None),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_sim-cli"))
            .args(args)
            .output()
            .unwrap();
        if let Some(value) = expected {
            assert!(output.status.success());
            assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), value);
        } else {
            assert_eq!(output.status.code(), Some(2));
            assert!(String::from_utf8(output.stderr).unwrap().contains("usage:"));
        }
    }
}

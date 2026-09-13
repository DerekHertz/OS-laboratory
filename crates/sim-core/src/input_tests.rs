use super::*;
use serde_json::json;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/reference/contracts/cases.json"
    ))
    .unwrap()
}
fn sample() -> Value {
    fixture()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "V02-workload-preserves-u64")
        .unwrap()["value"]
        .clone()
}
fn decode(v: &Value) -> Result<Workload> {
    Workload::decode(&serde_json::to_vec(v).unwrap())
}

#[test]
fn t02_program_workload_fixture_parity() {
    let mut checked = 0;
    for case in fixture()["cases"].as_array().unwrap() {
        let bytes = serde_json::to_vec(&case["value"]).unwrap();
        let actual = match case["target"].as_str().unwrap() {
            "program" => Program::decode(&bytes).is_ok(),
            "workload" => Workload::decode(&bytes).is_ok(),
            _ => continue, // command routing and reply validation remain T07
        };
        assert_eq!(actual, case["valid"].as_bool().unwrap(), "{}", case["id"]);
        checked += 1;
    }
    assert_eq!(checked, 8); // exactly the program/workload-targeted T02 fixtures
    let v = sample();
    let w = decode(&v).unwrap();
    assert_eq!(w.arrival(0), Some(u64::MAX));
    assert_eq!(w.control_budget, u64::MAX);
    assert_eq!(w.source(), &v);
}

#[test]
fn integer_lexical_range_and_parameter_boundaries() {
    for raw in [
        json!(0),
        json!("00"),
        json!("+1"),
        json!("-1"),
        json!("1e3"),
        json!(" 1"),
        json!("18446744073709551616"),
    ] {
        let mut v = sample();
        v["seed"] = raw;
        assert_eq!(decode(&v).unwrap_err(), InputError::InvalidCommand);
    }
    for value in [0, 9_007_199_254_740_993, u64::MAX] {
        let mut v = sample();
        v["seed"] = json!(value.to_string());
        assert_eq!(decode(&v).unwrap().seed(), value);
    }
    let mut v = sample();
    v["programs"][0]["parameters"] = json!([{"id":"n","minimum":i64::MIN.to_string(),"default":"1","maximum":i64::MAX.to_string()}]);
    v["programs"][0]["blocks"][0]["duration"] = json!({"parameter":"n"});
    v["threads"][0]["parameters"] = json!({"n":i64::MAX.to_string()});
    assert!(decode(&v).is_ok());
    for raw in [
        json!("0"),
        json!("-0"),
        json!("-1"),
        json!("9223372036854775808"),
        json!(1),
    ] {
        v["threads"][0]["parameters"]["n"] = raw;
        assert!(decode(&v).is_err());
    }
    v["threads"][0]["parameters"] = json!({"unknown":"1"});
    assert!(decode(&v).is_err());
    for field in ["switchCost", "controlOperationBudgetPerTick"] {
        let mut v = sample();
        v["machine"][field] = json!("0");
        assert_eq!(decode(&v).is_ok(), field == "switchCost");
    }
}

#[test]
fn compatibility_precedence_and_closed_shapes() {
    let mut v = sample();
    for field in [
        "schemaVersion",
        "modelVersion",
        "engineVersion",
        "randomAlgorithmVersion",
    ] {
        v[field] = json!("future");
    }
    for (field, expected, value) in [
        (
            "schemaVersion",
            InputError::UnsupportedSchema,
            "os-lab-workload/1",
        ),
        (
            "modelVersion",
            InputError::UnsupportedModel,
            "sched.m1/revision-1",
        ),
        (
            "engineVersion",
            InputError::UnsupportedEngine,
            "sim-engine.0.1.0",
        ),
        (
            "randomAlgorithmVersion",
            InputError::UnsupportedRandomAlgorithm,
            "none.v1",
        ),
    ] {
        assert_eq!(decode(&v).unwrap_err(), expected);
        v[field] = json!(value);
    }
    assert!(decode(&v).is_ok());
    v["engineVersion"] = json!("future");
    v["threads"][0]["arrival"] = json!("invalid");
    assert_eq!(decode(&v).unwrap_err(), InputError::InvalidCommand);
    for pointer in [
        "",
        "/machine",
        "/policy",
        "/programs/0",
        "/programs/0/blocks/0",
        "/threads/0",
    ] {
        let mut v = sample();
        v.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(true));
        assert!(decode(&v).is_err(), "{pointer}");
    }
    let mut v = sample();
    v["programs"][0]["blocks"][0]["duration"] = json!({"literal":"1","parameter":"x"});
    assert!(decode(&v).is_err());
    assert!(Workload::decode(b"{} trailing").is_err());
}

#[test]
fn resource_limits_exact_depth_blocks_bytes_and_counts() {
    for depth in [64, 65] {
        let mut block = json!({"blockId":"done","op":"end"});
        for i in 0..depth {
            block = json!({"blockId":format!("r{i}"),"op":"repeat","count":{"literal":"1"},"body":[block]});
        }
        let mut v = sample();
        v["programs"][0]["blocks"] = json!([block]);
        assert_eq!(decode(&v).is_ok(), depth == 64);
    }
    for count in [10_000, 10_001] {
        let mut blocks: Vec<_> = (0..count - 1)
            .map(|i| json!({"blockId":format!("b{i}"),"op":"yield"}))
            .collect();
        blocks.push(json!({"blockId":"done","op":"end"}));
        let mut v = sample();
        v["programs"][0]["blocks"] =
            json!([{"blockId":"outer","op":"repeat","count":{"literal":"1"},"body":blocks}]);
        // repeat itself is recursively counted, so even body length 10,000 exceeds total.
        assert_eq!(decode(&v).unwrap_err(), InputError::ResourceLimit);
    }
    let mut v = sample();
    let mut blocks: Vec<_> = (0..9_999)
        .map(|i| json!({"blockId":format!("b{i}"),"op":"yield"}))
        .collect();
    blocks.push(json!({"blockId":"done","op":"end"}));
    v["programs"][0]["blocks"] = json!(blocks);
    assert!(decode(&v).is_ok());
    let v = sample();
    let mut bytes = serde_json::to_vec(&v).unwrap();
    bytes.resize(MAX_BYTES, b' ');
    assert!(Workload::decode(&bytes).is_ok());
    bytes.push(b' ');
    assert_eq!(
        Workload::decode(&bytes).unwrap_err(),
        InputError::ResourceLimit
    );
    for length in [128, 129] {
        let mut v = sample();
        v["programs"][0]["name"] = json!("😀".repeat(length));
        assert_eq!(decode(&v).is_ok(), length == 128);
    }
    for cores in [0, 1, 64, 65] {
        let mut v = sample();
        v["machine"]["cores"] = json!(cores);
        assert_eq!(decode(&v).is_ok(), (1..=64).contains(&cores));
    }
    for (number, valid) in [("1.0", true), ("1e0", true), ("1.5", false)] {
        let raw = serde_json::to_string(&sample())
            .unwrap()
            .replace("\"cores\":2", &format!("\"cores\":{number}"));
        assert_eq!(
            Workload::decode(raw.as_bytes()).is_ok(),
            valid,
            "cores {number}"
        );
    }
    let mut v = sample();
    v["programs"][0]["parameters"] = json!(
        (0..129)
            .map(|i| json!({"id":format!("n{i}"),"minimum":"0","default":"0","maximum":"0"}))
            .collect::<Vec<_>>()
    );
    assert_eq!(decode(&v).unwrap_err(), InputError::ResourceLimit);
}

#[test]
fn resource_limit_preflight_precedes_unrelated_shape_errors() {
    let mut oversized_parameters = sample();
    oversized_parameters["programs"][0]["parameters"] = json!(
        (0..129)
            .map(|i| json!({"id":format!("n{i}"),"minimum":"0","default":"0","maximum":"0"}))
            .collect::<Vec<_>>()
    );
    oversized_parameters["machine"]["machineId"] = json!("1invalid");
    assert_eq!(
        decode(&oversized_parameters).unwrap_err(),
        InputError::ResourceLimit
    );

    // The boundary remains exact: without a resource breach, the malformed ID
    // continues through ordinary workload validation.
    let mut at_limit = sample();
    at_limit["programs"][0]["parameters"] = json!(
        (0..128)
            .map(|i| json!({"id":format!("n{i}"),"minimum":"0","default":"0","maximum":"0"}))
            .collect::<Vec<_>>()
    );
    at_limit["machine"]["machineId"] = json!("1invalid");
    assert_eq!(decode(&at_limit).unwrap_err(), InputError::InvalidCommand);
}

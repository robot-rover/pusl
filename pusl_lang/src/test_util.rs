#![doc(hidden)]

use serde::{Serialize, Deserialize};
use std::{fs, fmt, io::{Write, Read}};
use xz::{read::XzDecoder, write::XzEncoder};

const RESOURCES: &str = "../resources/";

pub fn compare_test<T, F>(actual: &T, test_mod: &str, test_tag: &str, compare_fn: F) where for<'a> T: Serialize + Deserialize<'a>, F: FnOnce(&T, &T) {
    let mod_dir = format!("{RESOURCES}/{test_mod}");
    fs::create_dir_all(mod_dir).unwrap();
    let actual_file = format!("{RESOURCES}/{test_mod}/{test_tag}-actual.json.xz");
    let expect_file = format!("{RESOURCES}/{test_mod}/{test_tag}-expect.json.xz");

    let actual_json = serde_json::to_string_pretty(&actual).expect("Cannot serialize actual output json");
    let mut out_enc = XzEncoder::new(fs::File::create(actual_file).expect("Cannot open actual output json"), 9);
    out_enc.write_all(actual_json.as_bytes()).expect("Cannot write actual output json");

    let mut in_dec = XzDecoder::new(fs::File::open(expect_file).expect("Cannot find expected output json"));
    let mut expect_json = String::new();
    in_dec.read_to_string(&mut expect_json).expect("Cannot find expected output json");
    let expected = serde_json::from_str::<T>(&expect_json).expect("Cannot parse expected output json");

    compare_fn(&expected, actual);
}

pub fn compare_test_eq<T>(actual: &T, test_mod: &str, test_tag: &str) where for<'a> T: Serialize + Deserialize<'a> + PartialEq + fmt::Debug {
    compare_test(actual, test_mod, test_tag, |lhs: &T, rhs: &T| assert_eq!(lhs, rhs, "Compare Test failed {test_mod}/{test_tag}"));
}
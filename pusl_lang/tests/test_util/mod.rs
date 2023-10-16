#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    io,
};

const RESOURCES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources");
const TARGET: &str = env!("CARGO_TARGET_TMPDIR");

pub fn compare_test<T, F>(actual: &T, test_mod: &str, test_tag: &str, compare_fn: F)
where
    for<'a> T: Serialize + Deserialize<'a>,
    F: FnOnce(&T, &T),
{
    let expect_mod_dir = format!("{RESOURCES}/{test_mod}");
    fs::create_dir_all(expect_mod_dir).unwrap();
    let actual_mod_dir = format!("{TARGET}/{test_mod}");
    fs::create_dir_all(actual_mod_dir).unwrap();

    let expect_file = format!("{RESOURCES}/{test_mod}/{test_tag}-expect.json");
    let actual_file = format!("{TARGET}/{test_mod}/{test_tag}-actual.json");

    let mut out_stream = io::BufWriter::new(fs::File::create(actual_file).expect("Cannot open actual output json"));
    serde_json::to_writer_pretty(&mut out_stream, &actual).expect("Cannot serialize actual output json");

    let mut in_stream = io::BufReader::new(fs::File::open(expect_file).expect("Cannot find expected output json"));
    let expected =
        serde_json::from_reader::<_, T>(&mut in_stream).expect("Cannot parse expected output json");

    compare_fn(&expected, actual);
}

pub fn compare_test_eq<T>(actual: &T, test_mod: &str, test_tag: &str)
where
    for<'a> T: Serialize + Deserialize<'a> + PartialEq + fmt::Debug,
{
    compare_test(actual, test_mod, test_tag, |lhs: &T, rhs: &T| {
        assert_eq!(lhs, rhs, "Compare Test failed {test_mod}/{test_tag}")
    });
}

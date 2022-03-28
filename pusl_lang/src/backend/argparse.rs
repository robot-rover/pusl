use super::object::Value;
use std::{convert::TryFrom, fmt::Debug, ops::RangeBounds};

fn validate_num_args<R>(expected: R, actual: usize)
where
    R: RangeBounds<usize> + Debug,
{
    if !expected.contains(&actual) {
        panic!(
            "wrong number of arguments: expected {:?}, got {}",
            expected, actual
        )
    }
}

pub fn convert_arg<T: TryFrom<Value>>(value: Value, arg_index: usize) -> T {
    if let Ok(converted) = T::try_from(value) {
        converted
    } else {
        panic!("Arg {} is the wrong type", arg_index)
    }
}

pub fn parse_option<A>(args: Vec<Value>) -> Option<A>
where
    A: TryFrom<Value>,
{
    validate_num_args(0..=1, args.len());
    let mut arg_iter = args.into_iter();
    let arg = arg_iter.next().map(|value| convert_arg(value, 0));
    arg
}

pub fn parse0(args: Vec<Value>) {
    validate_num_args(0..=0, args.len());
}

pub fn parse1<A>(args: Vec<Value>) -> A
where
    A: TryFrom<Value>,
{
    validate_num_args(1..=1, args.len());
    let mut arg_iter = args.into_iter();
    let arg0 = arg_iter.next().unwrap();
    let arg0 = convert_arg(arg0, 0);
    arg0
}

pub fn parse2<A, B>(args: Vec<Value>) -> (A, B)
where
    A: TryFrom<Value>,
    B: TryFrom<Value>,
{
    validate_num_args(2..=2, args.len());
    let mut arg_iter = args.into_iter();
    let arg0 = arg_iter.next().unwrap();
    let arg0 = convert_arg(arg0, 0);
    let arg1 = arg_iter.next().unwrap();
    let arg1 = convert_arg(arg1, 1);
    (arg0, arg1)
}

pub fn parse3<A, B, C>(args: Vec<Value>) -> (A, B, C)
where
    A: TryFrom<Value>,
    B: TryFrom<Value>,
    C: TryFrom<Value>,
{
    validate_num_args(3..=3, args.len());
    let mut arg_iter = args.into_iter();
    let arg0 = arg_iter.next().unwrap();
    let arg0 = convert_arg(arg0, 0);
    let arg1 = arg_iter.next().unwrap();
    let arg1 = convert_arg(arg1, 1);
    let arg2 = arg_iter.next().unwrap();
    let arg2 = convert_arg(arg2, 2);
    (arg0, arg1, arg2)
}

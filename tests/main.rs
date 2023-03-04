use serde::Serialize;
use serde_derive::{Deserialize, Serialize};

use wolfram_expr::WolframSerializer;

#[derive(Serialize, Deserialize)]
pub struct TestEmpty;
#[derive(Serialize, Deserialize)]
pub struct TestEmptyTuple();
#[derive(Serialize, Deserialize)]
pub struct TestEmptyDict {}

#[derive(Serialize, Deserialize)]
pub struct Point2(f64, f64);

#[derive(Serialize, Deserialize)]
pub struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, Deserialize)]
pub struct TestBody<'a> {
    empty: TestEmpty,
    empty_tuple: TestEmptyTuple,
    empty_dict: TestEmptyDict,
    unit: (),
    bool: bool,
    int: i64,
    c: char,
    str: &'a str,
    string: String,
    bytes: &'a [u8],
    vector: Vec<u8>,
    point1: (f64,),
    point2: Point2,
    point3: Point3,
}

#[test]
fn test_serialize() {
    let serializer = WolframSerializer {};
    let str = "ref string";
    let bytes = &[1, 2, 3, 4, 5];
    let input = TestBody {
        empty: TestEmpty,
        empty_tuple: TestEmptyTuple(),
        empty_dict: TestEmptyDict {},
        unit: (),
        bool: true,
        int: 42,
        c: 'c',
        str,
        string: "own string".to_string(),
        bytes,
        vector: bytes.to_vec(),
        point1: (1.0,),
        point2: Point2(1.0, 2.0),
        point3: Point3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    };
    let out = input.serialize(&serializer).unwrap();
    println!("{:?}", out);
}

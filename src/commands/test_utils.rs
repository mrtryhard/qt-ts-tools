use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::Serialize;

use crate::ts::TSNode;

pub fn node_to_formatted_string(node: &TSNode) -> String {
    let mut buf = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE TS>\n".to_string();
    let mut ser = quick_xml::se::Serializer::new(&mut buf);
    ser.indent(' ', 4).expand_empty_elements(true);
    node.serialize(ser).expect("Nodes are serializable");
    buf
}

pub fn read_test_file(filename: &str) -> String {
    let mut buf = String::new();
    let _ = File::open(PathBuf::new().join("./test_data").join(filename))
        .expect("Data file is readable")
        .read_to_string(&mut buf)
        .expect("Output to string");
    buf.replace('\r', "")
}

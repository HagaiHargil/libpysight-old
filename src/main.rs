#![feature(uniform_paths)]
use rread_lst;

use rread_lst::parsing::DataLine;
use std::collections::HashMap;

fn main() {
    let mut starts_map = HashMap::new();
    starts_map.insert("trial1.lst", 2usize);
    starts_map.insert("4-byte006.lst", 1480usize);
    starts_map.insert("power_40p7_512unidir_gain900_thresh18mv_start_pmt1_stop1_lines_calcium_002.lst",
                      1554usize);
    let fname = "4-byte006.lst";
    let start_of_data = starts_map[fname];
    let range = 512u64;
    let timepatch = "5";
    let channel_map = vec![0, 0, 0, 0, 0, 1];
    let res = rread_lst::analyze_lst(fname, start_of_data, range, timepatch, channel_map).unwrap();
    println!("{:?}", res);
    // println!("{:?}", res[&0]);
}

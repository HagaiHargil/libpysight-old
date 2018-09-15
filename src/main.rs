#![feature(rust_2018_preview, uniform_paths)]
extern crate rread_lst;
extern crate filebuffer;
extern crate data_encoding;

use rread_lst::reading::analyze_lst;
use rread_lst::from_playground::par_main;
use std::collections::HashMap;
use std::str;
use filebuffer::FileBuffer;
use data_encoding::HEXLOWER;


fn main() {
    let mut starts_map = HashMap::new();
    starts_map.insert("trial1.lst", 2usize);
    starts_map.insert("4-byte006.lst", 1480usize);
    starts_map.insert("power_40p7_512unidir_gain900_thresh18mv_start_pmt1_stop1_lines_calcium_002.lst",
                      1554usize);
    let fname = "4-byte006.lst";
    let _start_of_data = starts_map[fname];
    let _range = 512u64;
    let _timepatch = "5";
    let _channel_map = vec![0, 0, 0, 0, 0, 1];
    // let res = analyze_lst(fname, start_of_data, range, timepatch, channel_map).unwrap();
    // println!("{:?}", res[&6]);
    // println!("{:?}", res[&0]);
    par_main();

}

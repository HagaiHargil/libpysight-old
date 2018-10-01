// #![feature(uniform_paths)]
// use rread_lst;

// use std::collections::HashMap;

// fn main() {
//     let mut starts_map = HashMap::new();
//     starts_map.insert("trial1.lst", 2usize);
//     starts_map.insert("4p5mW_850g_008.lst", 1565usize); // channels - 1, 6
//     starts_map.insert("Mouse_LPT_189kHz_62p_Penetrating_arteries_FOV2_20x_Zoom_512lines_512px_400um_higher_than_Nominal_depth_800nm_039.lst",
//                       1565usize); // channels - 2, 6
//     let fname = "4-byte006.lst";
//     let start_of_data = starts_map[fname];
//     let range = 512u64;
//     let timepatch = "5";
//     let channel_map = vec![0, 0, 0, 0, 0, 1];
//     let res = rread_lst::analyze_lst_u16(fname, start_of_data, range, timepatch, channel_map);
//     println!("{:?}", res);
//     // println!("{:?}", res[&0]);
// }

fn main() {}
extern crate csv;
extern crate bitreader;

use std::fs::File;
use std::io::SeekFrom;
use std::io::Seek;
use std::io::Read;
use self::csv::Reader;

pub struct DataLine {
    pub lost: bool,
    pub channel: u8,
    pub edge: bool,
    pub sweep: u32,
    pub time: u64,
}

pub fn main() -> Vec<DataLine> {
    let file_path = r"C:\Users\Hagai\Documents\GitHub\Multiscaler_Image_Generator\PMT1_Readings_one_sweep_equals_one_frame.lst";
    let start_of_data_pos: u64 = 1546;
    let range = 8u32;
    let timepatch = String::from("32");
    let returned_data_from_bytes = read_file_as_bytes(file_path, start_of_data_pos,
                                                      timepatch, range);

    returned_data_from_bytes
}

pub fn read_file_as_bytes(file_path: &str, start_of_data_pos: u64,
                          timepatch: String, range: u32) -> Vec<DataLine> {

    let mut f = File::open(file_path).unwrap();
    let _ = f.seek(SeekFrom::Start(start_of_data_pos));
    let data = csv::Reader::from_reader(f).has_headers(false);

    let processed_data = match timepatch.as_str() {
        "32" => timepatch_32(data, range),
        _ => panic!("Problem!")
    };

    processed_data
}

fn timepatch_32<R: Read>(mut data: Reader<R>, range: u32) -> Vec<DataLine> {
    let mut data_processed: Vec<DataLine> = Vec::new();
    let mut lost: bool;
    let mut sweep: u32;
    let mut time: u64;
    let mut edge: bool;
    let mut channel: u8;

    loop {
        match data.next_bytes() {
            csv::NextField::EndOfCsv => break,
            csv::NextField::EndOfRecord => {},
            csv::NextField::Error(err) => panic!(err),
            csv::NextField::Data(row) => {
                let mut reader = bitreader::BitReader::new(row);
                lost = reader.read_bool().unwrap();
                sweep = reader.read_u32(7).unwrap();
                time = reader.read_u64(36).unwrap() + ((range * sweep - 1) as u64);
                edge = reader.read_bool().unwrap();
                channel = reader.read_u8(3).unwrap();

                data_processed.push(DataLine { channel: channel, edge: edge, sweep: sweep,
                                               time: time, lost: lost })
            }
        }
    }

    data_processed
}
//pub fn read_file_as_str() -> Vec<DataLine> {
//    // Read the filename as an ASCII file
//
//    let file_path = r"C:\Users\Hagai\Documents\GitHub\Multiscaler_Image_Generator\PMT1_Readings_one_sweep_equals_one_frame.lst";
//    let start_of_data_pos: u64 = 1546;
//    const NUM_OF_LINES: usize = 10000;
//
//    let mut all_bin: &str = "1";
//    let hex_to_bin: HashMap<char, &'static str> = create_hash_map_1d();
//    let mut time: u64 = 1;
//
//    let mut data_processed = Vec::with_capacity(NUM_OF_LINES);
//    let mut f = File::open(file_path).unwrap();
//    let _ = f.seek(SeekFrom::Start(start_of_data_pos));
//
//    let mut data = csv::Reader::from_reader(f).has_headers(false);
//    for mut row in data.records().map(|r| r.unwrap()) {
//        // Code block #1
//        let all_bin = match hex_to_bin.get(&row[0].pop().unwrap()) {
//            Some(bin_value) => bin_value,
//            None => "9876"
//        };
//        let (edge, channel) = all_bin.split_at(1);
//
//        // Code block #2
//        // let (lost, sweep) = get_lost_sweep(&row[0][0..2]);
//
//
//        data_processed.push(DataLine {channel: channel.to_string(), edge: edge.to_string(), sweep: 1, time: 1 });
//    }
//    data_processed
//}
//
//fn create_hash_map_1d<'a>() -> HashMap<char, &'a str> {
//    let vec_of_letters = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
//    let mut hex_to_bin_map = HashMap::new();
//    let mut num = 0;
//    let mut midway = "a".to_string();
//    for (idx, hex_let1) in (&vec_of_letters).enumerate() {
//        midway = format!("{:04b}", idx);
//        hex_to_bin_map.insert(hex_let1, midway);
//    }
//
//    //let hex_to_bin_map = hex_to_bin_map;
//    hex_to_bin_map
//
//}

//fn create_hash_map_2d() -> HashMap<String, String> {
//    let vec_of_letters = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
//    let mut hex_to_bin_map = HashMap::new();
//    let mut num = 0;
//    let mut midway = "a".to_string();
//
//    for hex_let1 in &vec_of_letters {
//        for hex_let2 in &vec_of_letters {
//            midway = format!("{:08b}", num);
//            hex_to_bin_map.insert(hex_let1.to_string() + &hex_let2.to_string(), midway);
//            num = num + 1;
//        }
//
//    }
//    let hex_to_bin_map = hex_to_bin_map;
//    hex_to_bin_map
//}


//fn get_lost_sweep<'a>(row: &'a str) -> (String, u16) {
//    /// get a 2-byte &str from the .lst file and return the sweep and data lost bits
//
//    let bin_bytes: String = format!(&"{:b}", row);
//    let num: u16 = 2;
//
//    (bin_bytes, num)
//
//}
//


use std::fs;
// use rayon::prelude::*;
use std::collections::HashMap;
use filebuffer::FileBuffer;
use bitreader::BitReader;


pub struct DataLine {
    pub lost: u8,
    pub tag: u16,
    pub channel: u8,
    pub edge: bool,
    pub sweep: u16,
    pub time: u64,
}

pub fn main() -> Vec<DataLine> {
    // let file_path = r"C:\Users\Hagai\Documents\GitHub\rread_lst\TAG lens in fixed sample 3 - 62p 188kHz - zoom 4x - single 65.6 milliseconds sweep - 8 byte words - 1200 um ABOVE SAMPLE - 200 fps - unlocked to scanimage - BINARY LISTFILE - 300 mV threshold - 011.lst";
    let file_path = r"C:\Users\Hagai\Documents\GitHub\rread_lst\TAG lens in fixed sample 3 - 62p 188kHz - zoom 4x - single 10 milliseconds sweep - 8 byte words - 130 um DEEP - 200 fps - unlocked to scanimage - BINARY LISTFILE - 300 mV threshold - 009.lst";
    let start_of_data_pos: usize = 1572;
    let range = 8u64;
    let timepatch = String::from("5b");
    let returned_data_from_bytes = read_file_as_bytes(file_path, start_of_data_pos,
                                                      timepatch.as_str(), range);

    returned_data_from_bytes
}

fn read_file_as_bytes(file_path: &str, start_of_data_pos: usize,
                      timepatch: &str, range: u64) -> Vec<DataLine> {

    // Open the file and go to start-of-data position
    let data_including_headers = FileBuffer::open(file_path).unwrap();
    let data_size: usize = (fs::metadata(file_path).unwrap().len() - start_of_data_pos as u64) as usize;

    // Find the suitable bit order in the file
    let timepatch_map = create_timepatch_map();
    let bit_order: &[u8; 4] = timepatch_map.get(timepatch).unwrap();
    let mut chunk_size: u8 = bit_order.iter().sum();
    chunk_size = (chunk_size + 4) / 8;

    // Get data
    let processed_data;
    if String::from("f3") == timepatch {
        processed_data = iterate_over_f3(&data_including_headers[start_of_data_pos..], range,
                                               bit_order, chunk_size as usize, data_size);
    } else {
        processed_data = iterate_over_file(&data_including_headers[start_of_data_pos..], range,
                                               bit_order, chunk_size as usize, data_size);
    }
    processed_data
}

fn create_timepatch_map<'a>() -> HashMap<&'a str, [u8; 4]> {

    let mut timepatch_map = HashMap::new();

    // Array elements: Data lost, TAG bits, Sweep, Time (edge and channel are always 1 and 3)
    let array_for_32 = [1, 0, 7, 36];
    timepatch_map.insert("32", array_for_32);

    let array_for_f3 = [1, 16, 7, 36];
    //let array_for_f3 = [0, 0, 0, 0];
    timepatch_map.insert("f3", array_for_f3);

    let array_for_5b = [1, 15, 16, 28];
    timepatch_map.insert("5b", array_for_5b);

    timepatch_map
}

fn iterate_over_f3(data: &[u8], range: u64, bit_order: &[u8; 4], chunk_size: usize,
                     data_size: usize) -> Vec<DataLine> {
    let mut data_processed: Vec<DataLine> = Vec::with_capacity(data_size + 1);
    let mut lost: u8;
    let mut tag: u16;
    let mut sweep: u16;
    let mut time: u64;
    let mut edge: bool;
    let mut channel: u8;

    println!("chunk_size: {}, data_size: {}, data/chunk: {}", chunk_size, data_size,
             data_size/chunk_size);
    println!("Making sure we didn't lose anyone: {}", chunk_size * (data_size/chunk_size));

    let first_row = &data[0..chunk_size];
    println!("first row: {:?}", first_row);
    let second_row = &data[chunk_size..chunk_size*2];
    println!("second row: {:?}", second_row);
    let third_row = &data[chunk_size*2..chunk_size*3];
    println!("third row: {:?}", third_row);
    let fourth_row = &data[chunk_size*3..chunk_size*4];
    println!("fourth row: {:?}", fourth_row);

    let mut reader_f = BitReader::new(first_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
    let mut reader_f = BitReader::new(second_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
    let mut reader_f = BitReader::new(third_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
    let mut reader_f = BitReader::new(fourth_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
//    let bits_lost = reader_f.read_u64(1).unwrap();
//    let bits_tag = reader_f.read_u64(15).unwrap();
//    println!("Bits_lost: {:b}", bits_lost);
//    println!("Bits_tag: {:b}", bits_tag);

    for cur_data in data.chunks(chunk_size) {
        let mut reader = BitReader::new(cur_data);
        tag = reader.read_u16(bit_order[1]).unwrap();
        lost = reader.read_u8(bit_order[0]).unwrap();
        sweep = reader.read_u16(bit_order[2]).unwrap();
        time = reader.read_u64(bit_order[3]).unwrap();
        edge = reader.read_bool().unwrap();
        channel = reader.read_u8(3).unwrap();

        time = time  + (range * (sweep as u64)); // TODO: should be -1

        data_processed.push(DataLine { channel: channel, edge: edge, sweep: sweep,
                                       time: time, tag: tag, lost: lost });
    }
    data_processed

}

fn iterate_over_file(data: &[u8], range: u64, bit_order: &[u8; 4], chunk_size: usize,
                     data_size: usize) -> Vec<DataLine> {
    let mut data_processed: Vec<DataLine> = Vec::with_capacity(data_size + 1);
    let mut lost: u8;
    let mut tag: u16;
    let mut sweep: u16;
    let mut time: u64;
    let mut edge: bool;
    let mut channel: u8;

    println!("chunk_size: {}, data_size: {}, data/chunk: {}", chunk_size, data_size,
             data_size/chunk_size);
    println!("Making sure we didn't lose anyone: {}", chunk_size * (data_size/chunk_size));
    let first_row = &data[0..chunk_size];
    println!("first row: {:?}", first_row);
    let second_row = &data[chunk_size..chunk_size*2];
    println!("second row: {:?}", second_row);
    let third_row = &data[chunk_size*2..chunk_size*3];
    println!("third row: {:?}", third_row);
    let fourth_row = &data[chunk_size*3..chunk_size*4];
    println!("fourth row: {:?}", fourth_row);

    let mut reader_f = BitReader::new(first_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
    let mut reader_f = BitReader::new(second_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
    let mut reader_f = BitReader::new(third_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
    let mut reader_f = BitReader::new(fourth_row);
    let all_bits = reader_f.read_u64(64).unwrap();
    println!("All bits 1: {:b}", all_bits);
//    let bits_lost = reader_f.read_u64(1).unwrap();
//    let bits_tag = reader_f.read_u64(15).unwrap();
//    println!("Bits_lost: {:b}", bits_lost);
//    println!("Bits_tag: {:b}", bits_tag);

    for cur_data in data.chunks(chunk_size) {
        let mut reader = BitReader::new(cur_data);
        lost = reader.read_u8(bit_order[0]).unwrap();
        tag = reader.read_u16(bit_order[1]).unwrap();
        sweep = reader.read_u16(bit_order[2]).unwrap();
        time = reader.read_u64(bit_order[3]).unwrap();
        edge = reader.read_bool().unwrap();
        channel = reader.read_u8(3).unwrap();

        time = time  + (range * (sweep as u64)); // TODO: should be -1

        data_processed.push(DataLine { channel: channel, edge: edge, sweep: sweep,
                                       time: time, tag: tag, lost: lost });
    }
    data_processed
}

//    loop {
//        match data.next_bytes() {
//            csv::NextField::EndOfCsv => break,
//            csv::NextField::EndOfRecord => {},
//            csv::NextField::Error(err) => panic!(err),
//            csv::NextField::Data(row) => {
//                let mut reader = bitreader::BitReader::new(row);
//                //channel = reader.read_u8(3).unwrap();
//                channel = 1;
//                println!("row: {:?}", row);
//                edge = reader.read_bool().unwrap();
//                time = reader.read_u64(bit_order[3]).unwrap();
//                lost = reader.read_u8(bit_order[0]).unwrap();
//                sweep = reader.read_u16(bit_order[2]).unwrap();
//                tag = reader.read_u16(bit_order[1]).unwrap();
//                time = time  + (range * (sweep as u64)); // TODO: should be -1
//
//                data_processed.push(DataLine { channel: channel, edge: edge, sweep: sweep,
//                                               time: time, tag: tag, lost: lost })
//            }
//        }
//    }

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


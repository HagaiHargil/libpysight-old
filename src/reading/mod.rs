use std::fs;
// use rayon::prelude::*;
use std::collections::HashMap;
use filebuffer::FileBuffer;
use bitreader::BitReader;


pub struct DataLine {
    pub lost: u8,
    pub tag: u16,
//    pub channel: u8,
    pub edge: bool,
    pub sweep: u16,
    pub time: u64,
}

pub fn main() -> HashMap<u8, Vec<DataLine>> {
    let file_path = r"C:\Users\Hagai\Documents\GitHub\rread_lst\TAG lens in fixed sample 3 - 62p 188kHz - zoom 4x - single 65.6 milliseconds sweep - 8 byte words - 1200 um ABOVE SAMPLE - 200 fps - unlocked to scanimage - BINARY LISTFILE - 300 mV threshold - 013.lst";
    // let file_path = r"C:\Users\Hagai\Documents\GitHub\rread_lst\TAG lens in fixed sample 3 - 62p 188kHz - zoom 4x - single 10 milliseconds sweep - 8 byte words - 130 um DEEP - 200 fps - unlocked to scanimage - BINARY LISTFILE - 300 mV threshold - 009.lst";
    let start_of_data_pos: usize = 1578;
    let range = 8u64;
    let timepatch = String::from("f3");
    let returned_data_from_bytes = read_file_as_bytes(file_path, start_of_data_pos,
                                                      timepatch.as_str(), range);

    returned_data_from_bytes
}

fn read_file_as_bytes(file_path: &str, start_of_data_pos: usize,
                      timepatch: &str, range: u64) -> HashMap<u8, Vec<DataLine>> {

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
    timepatch_map.insert("f3", array_for_f3);

    let array_for_5b = [1, 15, 16, 28];
    timepatch_map.insert("5b", array_for_5b);

    timepatch_map
}

fn create_channel_map(data_size: usize) -> HashMap<u8, Vec<DataLine>>{
    let mut channel_map = HashMap::new();

    for idx in 0..8 {
        channel_map.insert(idx as u8, Vec::with_capacity(data_size + 1));
    }

    channel_map
}

fn iterate_over_f3(data: &[u8], range: u64, bit_order: &[u8; 4], chunk_size: usize,
                     data_size: usize) -> HashMap<u8, Vec<DataLine>> {
    let mut lost: u8;
    let mut tag: u16;
    let mut sweep: u16;
    let mut time: u64;
    let mut edge: bool;
    let mut reversed_vec = Vec::with_capacity(chunk_size + 1);
    let mut map_of_data = create_channel_map(data_size);

    for cur_data in data.chunks(chunk_size) {
        reversed_vec.truncate(0);
        reversed_vec.extend(cur_data.iter().rev());
        let mut reader = BitReader::new(&reversed_vec);
        tag = reader.read_u16(bit_order[1]).unwrap();
        lost = reader.read_u8(bit_order[0]).unwrap();
        sweep = reader.read_u16(bit_order[2]).unwrap();
        time = reader.read_u64(bit_order[3]).unwrap();
        edge = reader.read_bool().unwrap();

        time = time + range * ((sweep) as u64); // TODO: should be -1

        // Populate a hashmap, each key being an input channel and the values are a vector
        // of DataLines
        map_of_data.get_mut(&reader.read_u8(3).unwrap()).unwrap()
            .push(DataLine { edge: edge, sweep: sweep,
                             time: time, tag: tag, lost: lost });
    }
    map_of_data
}

fn iterate_over_file(data: &[u8], range: u64, bit_order: &[u8; 4], chunk_size: usize,
                     data_size: usize) -> HashMap<u8, Vec<DataLine>> {

    let mut lost: u8;
    let mut tag: u16;
    let mut sweep: u16;
    let mut time: u64;
    let mut edge: bool;
    let mut reversed_vec = Vec::with_capacity(chunk_size + 1);
    let mut map_of_data = create_channel_map(data_size);

    for cur_data in data.chunks(chunk_size) {
        reversed_vec.truncate(0);
        reversed_vec.extend(cur_data.iter().rev());
        let mut reader = BitReader::new(&reversed_vec);
        lost = reader.read_u8(bit_order[0]).unwrap();
        tag = reader.read_u16(bit_order[1]).unwrap();
        sweep = reader.read_u16(bit_order[2]).unwrap();
        time = reader.read_u64(bit_order[3]).unwrap();
        edge = reader.read_bool().unwrap();
        time = time  + (range * ((sweep) as u64)); // TODO: should be -1

        // Populate a hashmap, each key being an input channel and the values are a vector
        // of DataLines
        map_of_data.get_mut(&reader.read_u8(3).unwrap()).unwrap()
            .push(DataLine { edge: edge, sweep: sweep,
                             time: time, tag: tag, lost: lost });
    }
    map_of_data
}


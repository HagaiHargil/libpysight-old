use std::fs;
use rayon::prelude::*;
use std::collections::HashMap;
use std::str;
use filebuffer::FileBuffer;
use bitreader::BitReader;
use failure::Error;
use std::sync::Mutex;
use rayon::prelude::*;
use byteorder::{ReadBytesExt, LE};
use num_traits::sign::Unsigned;

const NUM_OF_INPUT_CHANNELS: usize = 6;


/// The basic struct with all of the information
/// contained in a single "line" of an .lst file
#[derive(Clone, Debug)]
#[repr(C)]
pub struct DataLine {
    lost: bool,
    tag: u16,
    edge: bool,
    time: u64,
}

impl DataLine {
    fn new(lost: bool, tag: u16, edge: bool, time: u64) -> DataLine {
        DataLine { lost, tag, edge, time }
    }
}

/// Populates a vector of mutex-controlled vectors with the valid active channels of the
/// experiment. Each channel is a vector, its size being `data_size + 1` if there was
/// any data in that channel. Otherwise, its size is 0. Each vector is also wrapped in a
/// mutex to allow for multi-threaded parsing.
/// Note - I don't use an Option<Mutex<Vec>>> here since I wasn't able to make it compile,
/// although it probably is the more ergonomic version.
fn create_channel_vec(data_size: usize, active_channels: Vec<u8>) -> Vec<Mutex<Vec<DataLine>>> {
    let vec_with_data = Vec::with_capacity(data_size + 1);
    let vec_empty_chan = Vec::with_capacity(0);
    let mut chans = Vec::with_capacity(NUM_OF_INPUT_CHANNELS);
    for is_active in active_channels.iter() {
        if is_active == &1u8 {
            chans.push(Mutex::new(vec_with_data.clone()));
        } else {
            chans.push(Mutex::new(vec_empty_chan.clone()));
        };
    };
    chans
}

fn create_channel_vec_seq(data_size: usize, active_channels: Vec<u8>) -> Vec<Vec<DataLine>> {
    let vec_with_data = Vec::with_capacity(data_size + 1);
    let vec_empty_chan = Vec::with_capacity(0);
    let mut chans = Vec::with_capacity(NUM_OF_INPUT_CHANNELS);
    for is_active in active_channels.iter() {
        if is_active == &1u8 {
            chans.push(vec_with_data.clone());
        } else {
            chans.push(vec_empty_chan.clone());
        };
    };
    chans
}

/// Return type of our main function
type LstReturn = fn(&[u8], u64, &[u8; 4], Vec<Mutex<Vec<DataLine>>>)
        -> Result<Vec<Mutex<Vec<DataLine>>>, Error>;

/// Maps a timepatch value to the function that parses it.
enum Timepatch {
    Tp0(LstReturn),
    Tp5(LstReturn),
    Tp1(LstReturn),
    Tp1a(LstReturn),
    Tp2a(LstReturn),
    Tp22(LstReturn),
    Tp32(LstReturn),
    Tp2(LstReturn),
    Tp5b(LstReturn),
    TpDb(LstReturn),
    Tpf3(LstReturn),
    Tp43(LstReturn),
    Tpc3(LstReturn),
    Tp3(LstReturn),
}

impl Timepatch {
    fn new(tp: &str) -> Timepatch {
        match tp {
            "0" => Timepatch::Tp0(par_parse_no_sweep_2bytes),
            "5" => Timepatch::Tp5(par_parse_with_sweep_4bytes),
            "1" => Timepatch::Tp1(par_parse_no_sweep_4bytes),
            // "1a" => Timepatch::Tp1a(par_parse_with_sweep_u32),
            // "2a" => Timepatch::Tp2a(par_parse_with_sweep),
            // "22" => Timepatch::Tp22(parse_no_sweep),
            // "32" => Timepatch::Tp32(par_parse_with_sweep),
            // "2" => Timepatch::Tp2(parse_no_sweep),
            // "5b" => Timepatch::Tp5b(par_parse_with_sweep),
            // "Db" => Timepatch::TpDb(par_parse_with_sweep),
            // "f3" => Timepatch::Tpf3(parse_f3),
            "43" => Timepatch::Tp43(par_parse_no_sweep_8bytes),
            // "c3" => Timepatch::Tpc3(parse_no_sweep),
            // "3" => Timepatch::Tp3(parse_no_sweep),
            _ => panic!("Invalid timepatch value: {}", tp),
        }
    }
}

/// An array with four entries:
/// 0 - data lost
/// 1 - number of tag bits
/// 2 - number of sweep bits
/// 3 - number of time bits
struct TimepatchBits;

impl TimepatchBits {
    fn new(tp: &str) -> [u8; 4] {
        match tp {
            "0" => [0, 0, 0, 12],
            "5" => [0, 0, 8, 20],
            "1" => [0, 0, 0, 28],
            "1a" => [0, 0, 16, 28],
            "2a" => [0, 8, 8, 28],
            "22" => [0, 8, 0, 36],
            "32" => [1, 0, 7, 36],
            "2" => [0, 0, 0, 44],
            "5b" => [1, 15, 16, 28],
            "Db" => [0, 0, 16, 28],
            "f3" => [1, 16, 7, 36],
            "43" => [1, 15, 0, 44],
            "c3" => [0, 16, 0, 44],
            "3" => [1, 5, 0, 54],
            _ => panic!("Invalid timepatch value: {}.", tp),
        }
    }
}

fn to_bits_u16(bitarray: &[u8; 4]) -> [u16; 4] {
    let time_bits = "1".repeat(bitarray[3] as usize); 
    let time_bits = u16::from_str_radix(&time_bits, 2).unwrap();

    let sweep_bits = "1".repeat(bitarray[2] as usize);
    let sweep_bits = u16::from_str_radix(&sweep_bits, 2).unwrap_or(0);

    let tag_bits = "1".repeat(bitarray[1] as usize);
    let tag_bits = u16::from_str_radix(&tag_bits, 2).unwrap_or(0);

    let lost_bit = "1".repeat(bitarray[0] as usize);
    let lost_bit = u16::from_str_radix(&lost_bit, 2).unwrap_or(0);

    let bitmap: [u16; 4] = [lost_bit, tag_bits, sweep_bits, time_bits];
    bitmap  
}

fn to_bits_u32(bitarray: &[u8; 4]) -> [u32; 4] {
    let time_bits = "1".repeat(bitarray[3] as usize); 
    let time_bits = u32::from_str_radix(&time_bits, 2).unwrap();

    let sweep_bits = "1".repeat(bitarray[2] as usize);
    let sweep_bits = u32::from_str_radix(&sweep_bits, 2).unwrap_or(0);

    let tag_bits = "1".repeat(bitarray[1] as usize);
    let tag_bits = u32::from_str_radix(&tag_bits, 2).unwrap_or(0);

    let lost_bit = "1".repeat(bitarray[0] as usize);
    let lost_bit = u32::from_str_radix(&lost_bit, 2).unwrap_or(0);

    let bitmap: [u32; 4] = [lost_bit, tag_bits, sweep_bits, time_bits];
    bitmap  
}

fn to_bits_u64(bitarray: &[u8; 4]) -> [u64; 4] {
    let time_bits = "1".repeat(bitarray[3] as usize); 
    let time_bits = u64::from_str_radix(&time_bits, 2).unwrap();

    let sweep_bits = "1".repeat(bitarray[2] as usize);
    let sweep_bits = u64::from_str_radix(&sweep_bits, 2).unwrap_or(0);

    let tag_bits = "1".repeat(bitarray[1] as usize);
    let tag_bits = u64::from_str_radix(&tag_bits, 2).unwrap_or(0);

    let lost_bit = "1".repeat(bitarray[0] as usize);
    let lost_bit = u64::from_str_radix(&lost_bit, 2).unwrap_or(0);

    let bitmap: [u64; 4] = [lost_bit, tag_bits, sweep_bits, time_bits];
    bitmap  
}

/// Parse data in file if timepatch == "f3"
fn parse_f3(data: &[u8], range: u64, bit_order: &[u8; 4],
            map_of_data: HashMap<u8, Vec<DataLine>>) -> Result<HashMap<u8, Vec<DataLine>>, Error> {
    let mut lost: u8;
    let mut tag: u16;
    let mut sweep: u16;
    let mut time: u64;
    let mut edge: bool;
    let mut chan: u8;

    let mut chunk_size: usize = bit_order.iter().sum::<u8>() as usize;
    chunk_size = (chunk_size  + 4usize) / 8usize;
    let mut reversed_vec = Vec::with_capacity(chunk_size + 1usize);
    for cur_data in data.chunks(chunk_size) {
        reversed_vec.truncate(0);
        reversed_vec.extend(cur_data.iter().rev());
        let mut reader = BitReader::new(&reversed_vec);
        tag = reader.read_u16(bit_order[1]).expect("(f3) tag read problem.");
        lost = reader.read_u8(bit_order[0]).expect("(f3) lost read problem.");
        sweep = reader.read_u16(bit_order[2]).expect("(f3) sweep read problem.");
        time = reader.read_u64(bit_order[3]).expect("(f3) time read problem");
        time += range * (u64::from(sweep - 1));
        edge = reader.read_bool().expect("(f3) edge read problem.");
        chan = reader.read_u8(3).expect("channel read problem.");


        // Populate a hashmap, each key being an input channel and the values are a vector
        // of DataLines
        // map_of_data.get_mut(&chan).unwrap().push(DataLine::new(lost, tag, edge, sweep, time));
        };
    Ok(map_of_data)
}

/// Parse list files with a sweep counter
fn parse_with_sweep(data: &[u8], range: u64, bit_order: &[u8; 4],
                    mut map_of_data: Vec<Vec<DataLine>>) -> Result<Vec<Vec<DataLine>>, Error> {
    let mut lost: bool;
    let mut tag: u16;
    let mut sweep: u16;
    let mut time: u64;
    let mut edge: bool;
    let mut chan: u8;

    let mut chunk_size: usize = bit_order.iter().sum::<u8>() as usize;
    chunk_size = (chunk_size  + 4usize) / 8usize;
    let mut reversed_vec = Vec::with_capacity(chunk_size + 1usize);
    for cur_data in data.chunks(chunk_size) {
        println!("Baseline: {:?}", cur_data);
        reversed_vec.truncate(0);
        reversed_vec.extend(cur_data.iter().rev());
        println!("Reversed: {:?}", reversed_vec);
        let mut reader = BitReader::new(&reversed_vec);
        lost = reader.read_u8(bit_order[0]).expect("lost read problem.") == 1;
        tag = reader.read_u16(bit_order[1]).expect("tag read problem.");
        sweep = reader.read_u16(bit_order[2]).expect("sweep read problem.");
        time = reader.read_u64(bit_order[3]).expect("time read problem.");
        edge = reader.read_bool().expect("edge read problem.");
        chan = reader.read_u8(3).expect("channel read problem.");

        time += range * (u64::from(sweep - 1));
        println!("Time: {:?}", time);
        // Populate a hashmap, each key being an input channel and the values are a vector
        // of DataLines
        // map_of_data.get_mut(&chan).unwrap().push(DataLine::new(lost, tag, edge, time));
        map_of_data[(chan - 1) as usize].push(DataLine::new(lost, tag, edge, time));
    }

    Ok(map_of_data)
}

// /// Parse list files without a sweep counter
// fn parse_no_sweep(data: &[u8], _range: u64, bit_order: &[u8; 4],
//                   mut map_of_data: HashMap<u8, Vec<DataLine>>) -> Result<HashMap<u8, Vec<DataLine>>, Error> {
//     let mut lost: u8;
//     let mut tag: u16;
//     let mut sweep: u16;
//     let mut time: u64;
//     let mut edge: bool;
//     let mut chan: u8;


//     let mut chunk_size: usize = bit_order.iter().sum::<u8>() as usize;
//     chunk_size = (chunk_size  + 4usize) / 8usize;
//     let mut reversed_vec = Vec::with_capacity(chunk_size + 1usize);
//         for cur_data in data.chunks(chunk_size) {
//             reversed_vec.truncate(0);
//             reversed_vec.extend(cur_data.iter().rev());
//             // reversed_vec.extend(cur_data.iter());
//             let mut reader = BitReader::new(&reversed_vec);
//             lost = reader.read_u8(bit_order[0]).expect("lost read problem.");
//             tag = reader.read_u16(bit_order[1]).expect("tag read problem.");
//             sweep = reader.read_u16(bit_order[2]).expect("sweep read problem.");
//             time = reader.read_u64(bit_order[3]).expect("time read problem.");
//             edge = reader.read_bool().expect("edge read problem.");
//             chan = reader.read_u8(3).expect("channel read problem.");
//             // Populate a hashmap, each key being an input channel and the values are a vector
//             // of DataLines
//             println!("{:?}", (lost, tag, time, edge, chan));
//             map_of_data.get_mut(&chan)
//                 .expect("Chan not available in map")
//                 .push(DataLine::new(lost, tag, edge, sweep, time));
//         }

//     Ok(map_of_data)
// }

// pub fn par_main() {
//     let data: [u8; 12] = [246, 0, 0, 1, 246, 1, 0, 1, 230, 2, 0, 1];
//     let mut vec_of_channels = create_channel_vec(10usize, vec![0, 0, 0, 0, 0, 1]);
//     let res: Vec<_> = data
//         .par_chunks(4)
//         .filter_map(|mut line| if line != [0u8; 4] { 
//             line.read_u32::<LE>().ok()
//             } else { None })
//         .map(|mut line| {
//             let ch = ((line & 0b111) - 1) as usize;
//             let time = (line >> 4) & 0b111111111111;
//             println!("{:b}, {:b}", time, line);
//             let dl = DataLine::new(0, 0, false, 0, time.into());
//             vec_of_channels[ch]
//                 .lock()
//                 .expect("Mutex lock error")
//                 .push(dl);
//         }).collect();
//     println!("And finally: {:?}", vec_of_channels);
    
// }

/// Parse a list file for time patch "1"
fn par_parse_no_sweep_4bytes(data: &[u8], range: u64, bit_order: &[u8; 4],
                             mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u32(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u32::<LE>().ok()
            } else { None })
        .map(|mut line| {
            println!("{:?}", line);
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let mut time: u64 = (line & bitmap[3]).into();
            line = line >> bit_order[3]; // throw away "time" bits
            let dl = DataLine::new(false, 0, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}

/// Parse a list file for time patch "0"
fn par_parse_no_sweep_2bytes(data: &[u8], range: u64, bit_order: &[u8; 4],
                             mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u16(bit_order);
    let res: Vec<_> = data
        .par_chunks(2)
        .filter_map(|mut line| if line != [0u8; 2] { 
            line.read_u16::<LE>().ok()
            } else { None })
        .map(|mut line| {
            println!("{:?}", line);
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let mut time: u64 = (line & bitmap[3]).into();
            line = line >> bit_order[3]; // throw away "time" bits
            let dl = DataLine::new(false, 0, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}

/// Parse a list file for time patch "5"
fn par_parse_with_sweep_4bytes(data: &[u8], range: u64, bit_order: &[u8; 4],
                               mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let num_of_bytes_per_line = ((bit_order.iter().sum::<u8>() + 4) / 8) as usize;
    let bitmap = to_bits_u32(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u32::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
            let sweep: u16 = (line & bitmap[2]) as u16;
            time += range * (u64::from(sweep - 1));
            line = line >> bit_order[2]; // throw away "sweep" bits
            let tag: u16 = (line & bitmap[1]) as u16;
            line = line >> bit_order[1]; // throw away "tag" bits
            let lost: bool = (line & bitmap[0]) == 1;
            let dl = DataLine::new(lost, tag, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "5"
fn parse_with_sweep_4bytes(data: &[u8], range: u64, bit_order: &[u8; 4],
                           mut parsed_data: Vec<Vec<DataLine>>) 
    -> Result<Vec<Vec<DataLine>>, Error> {
    let num_of_bytes_per_line = ((bit_order.iter().sum::<u8>() + 4) / 8) as usize;
    let bitmap = to_bits_u32(bit_order);
    let res: Vec<_> = data
        .chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u32::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
            let sweep: u16 = (line & bitmap[2]) as u16;
            time += range * (u64::from(sweep - 1));
            line = line >> bit_order[2]; // throw away "sweep" bits
            let tag: u16 = (line & bitmap[1]) as u16;
            line = line >> bit_order[1]; // throw away "tag" bits
            let lost: bool = (line & bitmap[0]) == 1;
            let dl = DataLine::new(lost, tag, edge, time);
            parsed_data[ch].push(dl);
        }).collect();
    Ok(parsed_data)
}

/// Parse a list file for time patch "43"
fn par_parse_no_sweep_8bytes(data: &[u8], range: u64, bit_order: &[u8; 4],
                             mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u64(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u64::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let time: u64 = line & bitmap[3];
            line = line >> bit_order[3]; // throw away "time" bits
            let tag: u16 = (line & bitmap[1]) as u16;
            line = line >> bit_order[1]; // throw away "tag" bits
            let lost: bool = (line & bitmap[0]) == 1;
            let dl = DataLine::new(lost, tag, edge, time);
            parsed_data[ch].lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}

/// Parse a list file for time patch "43"
fn parse_with_sweep_8bytes(data: &[u8], range: u64, bit_order: &[u8; 4],
                           mut parsed_data: Vec<Vec<DataLine>>) 
    -> Result<Vec<Vec<DataLine>>, Error> {
    let num_of_bytes_per_line = ((bit_order.iter().sum::<u8>() + 4) / 8) as usize;
    let bitmap = to_bits_u64(bit_order);
    let res: Vec<_> = data
        .chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u64::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
            line = line >> bit_order[2]; // throw away "sweep" bits
            let tag: u16 = (line & bitmap[1]) as u16;
            line = line >> bit_order[1]; // throw away "tag" bits
            let lost: bool = (line & bitmap[0]) == 1;
            let dl = DataLine::new(lost, tag, edge, time);
            parsed_data[ch].push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse binary list files generated by a multiscaler.
/// Parameters:
/// fname - str
pub fn analyze_lst(fname: &str, start_of_data: usize, range: u64,
                   timepatch: &str, channel_map: Vec<u8>)
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {

    let data_with_headers = FileBuffer::open(fname).expect("bad file name");
    let data_size: usize = (fs::metadata(fname)?.len() - start_of_data as u64) as usize;
    let data = &data_with_headers[start_of_data..];
    // Open the file and convert it to a usable format

    let chan_map = create_channel_vec(data_size, channel_map);

    let tp_enum = Timepatch::new(timepatch);
    let processed_data = match tp_enum {
        Timepatch::Tp0(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp5(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp1(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp1a(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp2a(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp22(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp32(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp2(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp5b(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::TpDb(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tpf3(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp43(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tpc3(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        Timepatch::Tp3(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
    };
    processed_data

}

pub fn analyze_lst_par(no: i32) -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let fname = "1000nm_Pulsatile_Modulation_-9000mV_to_9500mV_1_sweep_each_32s_long009.lst";  //002
    let start_of_data = 1568usize;  // 1565
    let range = 80000u64;
    let timepatch = "43";
    let channel_map = vec![1, 0, 0, 0, 0, 1];

    let data_with_headers = FileBuffer::open(fname).expect("bad file name");
    let data_size: usize = (fs::metadata(fname)?.len() - start_of_data as u64) as usize;
    let data = &data_with_headers[start_of_data..];
    // Open the file and convert it to a usable format

    let chan_map = create_channel_vec(data_size, channel_map);

    let tp_enum = Timepatch::new(timepatch);
    let processed_data = match tp_enum {
        Timepatch::Tp43(func) => func(data, range, &TimepatchBits::new(timepatch), chan_map),
        _ => panic!()
    };
    processed_data
}

pub fn analyze_lst_seq(no: i32) -> Result<Vec<Vec<DataLine>>, Error> {
    
    let fname = "1000nm_Pulsatile_Modulation_-9000mV_to_9500mV_1_sweep_each_32s_long009.lst"; // 002
    let start_of_data = 1568usize;  // 1565
    let range = 80000u64;
    let timepatch = "43";
    let channel_map = vec![1, 0, 0, 0, 0, 1];

    let data_with_headers = FileBuffer::open(fname).expect("bad file name");
    let data_size: usize = (fs::metadata(fname)?.len() - start_of_data as u64) as usize;
    let data = &data_with_headers[start_of_data..];
    // Open the file and convert it to a usable format

    let chan_map = create_channel_vec_seq(data_size, channel_map.to_vec());

    let tp_enum = TimepatchBits::new(timepatch);
    let processed_data = parse_with_sweep_8bytes(data, range, &tp_enum, chan_map);
    processed_data

}

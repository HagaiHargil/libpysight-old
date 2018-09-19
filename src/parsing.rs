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

use helper_funcs::*;

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
            "0" => Timepatch::Tp0(parse_0),
            "5" => Timepatch::Tp5(parse_5),
            "1" => Timepatch::Tp1(parse_1),
            "1a" => Timepatch::Tp1a(parse_1a),
            "2a" => Timepatch::Tp2a(parse_2a),
            "22" => Timepatch::Tp22(parse_22),
            "32" => Timepatch::Tp32(parse_32),
            "2" => Timepatch::Tp2(parse_2),
            "5b" => Timepatch::Tp5b(parse_5b),
            "Db" => Timepatch::TpDb(parse_Db),
            "f3" => Timepatch::Tpf3(parse_f3),
            "43" => Timepatch::Tp43(parse_43),
            "c3" => Timepatch::Tpc3(parse_c3),
            "3" => Timepatch::Tp3(parse_3),
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

/// Parse a list file for time patch "1"
fn parse_1(data: &[u8], range: u64, bit_order: &[u8; 4],
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
fn parse_0(data: &[u8], range: u64, bit_order: &[u8; 4],
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
fn parse_5(data: &[u8], range: u64, bit_order: &[u8; 4],
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


/// Parse a list file for time patch "1a"
fn parse_1a(data: &[u8], range: u64, bit_order: &[u8; 4],
                           mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u64(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u48::<LE>().ok()
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
            let dl = DataLine::new(false, 0, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "2a"
fn parse_2a(data: &[u8], range: u64, bit_order: &[u8; 4],
            mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u64(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u48::<LE>().ok()
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
            let dl = DataLine::new(false, tag, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "22"
fn parse_22(data: &[u8], range: u64, bit_order: &[u8; 4],
            mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u64(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u48::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
            let tag: u16 = (line & bitmap[1]) as u16;
            let dl = DataLine::new(false, tag, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "5"
fn parse_32(data: &[u8], range: u64, bit_order: &[u8; 4],
            mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u64(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u48::<LE>().ok()
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
            let lost: bool = (line & bitmap[0]) == 1;
            let dl = DataLine::new(lost, 0, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "2"
fn parse_2(data: &[u8], range: u64, bit_order: &[u8; 4],
           mut parsed_data: Vec<Mutex<Vec<DataLine>>>) 
    -> Result<Vec<Mutex<Vec<DataLine>>>, Error> {
    let bitmap = to_bits_u64(bit_order);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u48::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = ((line & 0b111) - 1) as usize;
            line = line >> 3;  // throw away "channel" bits
            let edge = (line & 0b1) == 1;
            line = line >> 1;  // throw away "edge" bit
            let mut time: u64 = (line & bitmap[3]) as u64;
            let dl = DataLine::new(false, 0, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "5b"
fn parse_5b(data: &[u8], range: u64, bit_order: &[u8; 4],
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

/// Parse a list file for time patch "Db"
fn parse_Db(data: &[u8], range: u64, bit_order: &[u8; 4],
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
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
            let sweep: u16 = (line & bitmap[2]) as u16;
            time += range * (u64::from(sweep - 1));
            let dl = DataLine::new(0, 0, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}

/// Parse a list file for time patch "f3"
fn parse_f3(data: &[u8], range: u64, bit_order: &[u8; 4],
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
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
            let sweep: u16 = (line & bitmap[2]) as u16;
            time += range * (u64::from(sweep - 1));
            line = line >> bit_order[2]; // throw away "sweep" bits
            let lost: bool = (line & bitmap[0]) == 1;
            line = line >> bit_order[0]  // throw away lost bit
            let tag: u16 = (line & bitmap[1]) as u16;
            
            let dl = DataLine::new(lost, tag, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "43"
fn parse_43(data: &[u8], range: u64, bit_order: &[u8; 4],
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
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
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


/// Parse a list file for time patch "c3"
fn parse_c3(data: &[u8], range: u64, bit_order: &[u8; 4],
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
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
            let tag: u16 = (line & bitmap[1]) as u16;
            let dl = DataLine::new(false, tag, edge, time);
            parsed_data[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    Ok(parsed_data)
}


/// Parse a list file for time patch "3"
fn parse_3(data: &[u8], range: u64, bit_order: &[u8; 4],
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
            let mut time: u64 = (line & bitmap[3]) as u64;
            line = line >> bit_order[3]; // throw away "time" bits
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
fn seq_parse_5(data: &[u8], range: u64, bit_order: &[u8; 4],
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
fn seq_parse_43(data: &[u8], range: u64, bit_order: &[u8; 4],
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

/// Mock implementation of the parsing function that uses parallel
/// execution. Used for benchmarking.
#[cfg(test)]
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

/// Mock implementation of the parsing function that uses sequential,
/// instead of parallel, parsing. Used for benchmarking.
#[cfg(test)]
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

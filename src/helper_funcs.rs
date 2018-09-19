use std::sync::Mutex;

use crate::parsing::{NUM_OF_INPUT_CHANNELS, DataLine};


/// Populates a vector of mutex-controlled vectors with the valid active channels of the
/// experiment. Each channel is a vector, its size being `data_size + 1` if there was
/// any data in that channel. Otherwise, its size is 0. Each vector is also wrapped in a
/// mutex to allow for multi-threaded parsing.
/// Note - I don't use an Option<Mutex<Vec>>> here since I wasn't able to make it compile,
/// although it probably is the more ergonomic version.
pub fn create_channel_vec(data_size: usize, active_channels: Vec<u8>) -> Vec<Mutex<Vec<DataLine>>> {
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

pub fn create_channel_vec_seq(data_size: usize, active_channels: Vec<u8>) -> Vec<Vec<DataLine>> {
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


pub fn to_bits_u16(bitarray: &[u8; 4]) -> [u16; 4] {
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

pub fn to_bits_u32(bitarray: &[u8; 4]) -> [u32; 4] {
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

pub fn to_bits_u64(bitarray: &[u8; 4]) -> [u64; 4] {
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
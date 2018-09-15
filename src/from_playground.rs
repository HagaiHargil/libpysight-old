use rayon; // 1.0.2

use std::sync::{Arc, Mutex};
use std::ops::{Index, IndexMut};
use rayon::prelude::*;
use byteorder::{ReadBytesExt, LE};

const NUM_OF_INPUT_CHANNELS: usize = 6;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct DataLine {
    lost: u8,
    tag: u16,
    edge: bool,
    sweep: u16,
    time: u64,
}

impl DataLine {
    fn new(lost: u8, tag: u16, edge: bool, sweep: u16, time: u64) -> Self {
        DataLine { lost, tag, edge, sweep, time }
    }
}

/// Populates a vector of mutex-controlle vectors with the valid active channels of the
/// experiment. Each channel is a vector, its size being `data_size + 1`,
/// while it's also wrapped in an `Option` in case it wasn't active and in a
/// `Mutex` so that we could write data to it in parallel.
fn create_channel_struct(data_size: usize, active_channels: Vec<u8>) -> Vec<Mutex<Vec<DataLine>>> {
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


pub fn par_main() {
    let data: [u8; 12] = [246, 0, 0, 1, 246, 1, 0, 1, 230, 2, 0, 1];
    let mut vec_of_channels = create_channel_struct(10usize, vec![0, 0, 0, 0, 0, 1]);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u32::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = ((line & 0b111) - 1) as usize;
            let time = (line >> 4) & 0b111111111111;
            println!("{:b}, {:b}", time, line);
            let dl = DataLine::new(0, 0, false, 0, time.into());
            vec_of_channels[ch]
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    println!("And finally: {:?}", vec_of_channels);
    
}
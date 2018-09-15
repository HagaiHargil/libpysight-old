extern crate rayon; // 1.0.2

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

#[derive(Debug)]
pub struct InputChannels {
    ch1: Option<Mutex<Vec<DataLine>>>,
    ch2: Option<Mutex<Vec<DataLine>>>,
    ch3: Option<Mutex<Vec<DataLine>>>,
    ch4: Option<Mutex<Vec<DataLine>>>,
    ch5: Option<Mutex<Vec<DataLine>>>,
    ch6: Option<Mutex<Vec<DataLine>>>,
}

impl InputChannels {
    pub fn new(ch1: Option<Mutex<Vec<DataLine>>>, ch2: Option<Mutex<Vec<DataLine>>>,
           ch3: Option<Mutex<Vec<DataLine>>>, ch4: Option<Mutex<Vec<DataLine>>>,
           ch5: Option<Mutex<Vec<DataLine>>>, ch6: Option<Mutex<Vec<DataLine>>>) -> Self {
               InputChannels { ch1, ch2, ch3, ch4, ch5, ch6 }
           }    
}

impl Index<u8> for InputChannels {
    type Output = Option<Mutex<Vec<DataLine>>>;
    
    fn index<'a>(&'a self, index: u8) -> &'a Option<Mutex<Vec<DataLine>>> {
        match index {
               1 => &self.ch1,
               2 => &self.ch2,
               3 => &self.ch3,
               4 => &self.ch4,
               5 => &self.ch5,
               6 => &self.ch6,
               _ => panic!("Unexpected non_mut index: {}", index),
        }
    }
}

impl IndexMut<u8> for InputChannels {
    fn index_mut<'a>(&'a mut self, index: u8) -> &'a mut Option<Mutex<Vec<DataLine>>> {
           match index {
               1 => &mut self.ch1,
               2 => &mut self.ch2,
               3 => &mut self.ch3,
               4 => &mut self.ch4,
               5 => &mut self.ch5,
               6 => &mut self.ch6,
               _ => panic!("Unexpected mut index: {}", index),
           }
    }
}

/// Populates the InputChannels struct with the valid active channels of the
/// experiment. Each channel is a vector, its size being `data_size + 1`,
/// while it's also wrapped in an `Option` in case it wasn't active and in a
/// `Mutex` so that we could write data to it in parallel.
fn create_channel_struct(data_size: usize, active_channels: Vec<u8>) -> InputChannels {
    let vec = Vec::with_capacity(data_size + 1);
    let mut chans = Vec::with_capacity(NUM_OF_INPUT_CHANNELS);
    for is_active in active_channels.iter() {
        if is_active == &1u8 {
            chans.push(Some(Mutex::new(vec.clone())));
        } else {
            chans.push(None);
        };
    };
    
    InputChannels::new(chans.remove(0), chans.remove(0), chans.remove(0), 
                       chans.remove(0), chans.remove(0), chans.remove(0))
}


pub fn par_main() {
    let data: [u8; 12] = [246, 0, 0, 1, 246, 1, 0, 1, 230, 2, 0, 1];
    let mut ch_struct = create_channel_struct(10usize, vec![0, 0, 0, 0, 0, 1]);
    let res: Vec<_> = data
        .par_chunks(4)
        .filter_map(|mut line| if line != [0u8; 4] { 
            line.read_u32::<LE>().ok()
            } else { None })
        .map(|mut line| {
            let ch = (line & 0b111) as u8;
            let time = line & 0b11110000;
            let dl = DataLine::new(0, 0, false, 0, time.into());
            ch_struct.index_mut(ch).unwrap()
                .lock()
                .expect("Mutex lock error")
                .push(dl);
        }).collect();
    println!("And finally: {:?}", ch_struct);
    
}
#![feature(rust_2018_preview, uniform_paths)]
extern crate bitreader;
extern crate libc;
#[macro_use] extern crate failure;
#[macro_use] extern crate failure_derive;
extern crate filebuffer;
extern crate data_encoding;
extern crate rayon;
extern crate byteorder;

pub mod reading;
pub mod from_playground;

use reading::{analyze_lst, DataLine};
use libc::{uint64_t, c_char, c_void, size_t};
use std::ffi::CStr;
use std::{mem, slice};

/// Used to hold pointers to Vectors contains the analyzed data across FFI boundaries,
#[repr(C)]
pub struct DataPerChannel {
    ch_1: VecSlice,
    ch_2: VecSlice,
    ch_3: VecSlice,
    ch_4: VecSlice,
    ch_5: VecSlice,
    ch_6: VecSlice,
}

impl DataPerChannel {
    fn new(ch_1: VecSlice, ch_2: VecSlice, ch_3: VecSlice, ch_4: VecSlice, ch_5: VecSlice, ch_6: VecSlice)
        -> DataPerChannel {
        DataPerChannel { ch_1, ch_2, ch_3, ch_4, ch_5, ch_6 }
    }
}

/// A pair of pointer and data length to act as a vector in an FFI boundary.
#[repr(C)]
pub struct VecSlice {
    ptr: *mut u64,
    len: u64,
}

impl VecSlice {
    fn new(ptr: *mut u64, len: u64) -> VecSlice {
        VecSlice { ptr, len }
    }
}

/// Convert the input from Python into Rust data structures and call the main function
/// that reads and analyzes the `.lst` file
#[no_mangle]
pub extern "C" fn read_lst(file_path_py: *const c_char, start_of_data_pos: uint64_t,
                           range: uint64_t, timepatch_py: *const c_char) {
    
    let file_path_unsafe = unsafe {
        assert!(!file_path_py.is_null());
        CStr::from_ptr(file_path_py)
    };
    let file_path = file_path_unsafe.to_str().unwrap();

    let timepatch_unsafe = unsafe {
        assert!(!timepatch_py.is_null());
        CStr::from_ptr(timepatch_py)
    };
    let timepatch = timepatch_unsafe.to_str().unwrap();

    println!("{}, {}, {}", file_path, timepatch, range);


    // let s1 = VecSlice::new(p1, len1 };
    // ///
    // DataPerChannel::new(s1, s2, s3, s4, s5, s6)
}

#[no_mangle]
#[cfg(test)]
pub extern fn do_tuple_stuff(file_path_py: *const c_char, start_of_data_pos: uint64_t,
                             range: uint64_t, timepatch_py: *const c_char) -> Tuple {
    let file_path_unsafe = unsafe {
        assert!(!file_path_py.is_null());
        CStr::from_ptr(file_path_py)
    };
    let file_path = file_path_unsafe.to_str().unwrap();

    let timepatch_unsafe = unsafe {
        assert!(!timepatch_py.is_null());
        CStr::from_ptr(timepatch_py)
    };
    let timepatch = timepatch_unsafe.to_str().unwrap();

    println!("{}, {}, {}", file_path, timepatch, range);
    let mut data = reading::analyze_lst(String::from(file_path), start_of_data_pos as usize,
                                        range as u64, String::from(timepatch));

   let mut data = (vec![1, 2, 3], vec!['a', 'b', 'c'], vec!["1", "2", "3"]);
    let p0 = data.0.as_mut_ptr();
    let len0 = data.0.len() as u64;
    let p1 = data.1.as_mut_ptr();
    let len1 = data.0.len() as u64;
    let p2 = data.0.as_mut_ptr();
    let len2 = data.0.len() as u64;
    println!("{:?}", data.0[3]);

    unsafe {
        mem::forget(data);
    }

    let s0 = VecSlice { ptr: p0, len: len0 };
    let s1 = VecSlice { ptr: p1, len: len1 };
    let s2 = VecSlice { ptr: p2, len: len2 };
    DataPerChannel::new(s1, s2, s3, s4, s5, s6)
}

#[allow(dead_code)]
pub extern fn fix_linking_when_not_using_stdlib() { panic!() }

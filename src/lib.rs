extern crate bitreader;
extern crate filebuffer;
extern crate rayon;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

mod pre_reading;
pub mod reading;
mod processing;
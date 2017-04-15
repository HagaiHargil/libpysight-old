extern crate rread_lst;


fn main() {
    let data = rread_lst::reading::main();

    println!("Channel: {:b}", data[0].channel);
    println!("Edge: {:?}", data[0].edge);
    println!("Sweep: {:b}", data[0].sweep);
    println!("TAG: {:b}", data[0].tag);
    println!("Time: {:b}", data[0].time);
    println!("Num of entries: {:?}", data.len())

}
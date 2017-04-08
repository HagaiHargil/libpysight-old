extern crate rread_lst;

fn main() {
    let data = rread_lst::reading::main();

    println!("{:?}", data[0].channel);
    println!("{:?}", data[0].edge);
    println!("{:?}", data[0].sweep);
    println!("{:?}", data[0].time);
    println!("{:?}", data.len())

}
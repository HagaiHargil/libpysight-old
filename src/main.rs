extern crate rread_lst;


fn main() {
    let data = rread_lst::reading::main();

    //    let mut counter: u64 = 0;
//    let mut idx: u64 = 0;
//    let mut vec_of_sweeps = Vec::new();
//
//    for line in &data {
//        if line.sweep == 0 {
//            counter += 1;
//            vec_of_sweeps.push((line.sweep, idx, line.time, line.lost, line.tag, line.channel, line.edge));
//        }
//        idx += 1;
//    }
//
//    println!("Counter: {}", counter);
//    println!("Vec: {:?}, {:?}, {:?}, {:?}", vec_of_sweeps[0],
//    vec_of_sweeps[1], vec_of_sweeps[2], vec_of_sweeps[3]);
//    println!("Gen: {}, {}, {}", data[10].time, data[10].sweep, data[10].channel)


}
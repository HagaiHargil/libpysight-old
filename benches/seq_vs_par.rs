#[macro_use] extern crate criterion;

use criterion::{Criterion, Fun};
use rread_lst::reading::{analyze_lst_par, analyze_lst_seq};

fn get_config() -> Criterion {
    Criterion::default().nresamples(10)
                        .without_plots()
                        .sample_size(10)
}

fn pysight_benchmark(c: &mut Criterion) {
    let pysight_par = Fun::new("Parallel", 
        |b, t| b.iter(|| analyze_lst_par(*t)));

    let pysight_seq = Fun::new("Sequential", 
        |b, t| b.iter(|| analyze_lst_seq(*t)));
    let functions = vec!(pysight_par, pysight_seq);
    c.bench_functions("PySight", functions, 20i32);
}

criterion_group!(name = benches; 
                 config = get_config(); 
                 targets = pysight_benchmark);
criterion_main!(benches);
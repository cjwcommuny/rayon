use rand::{Rng, SeedableRng, XorShiftRng};
use rayon::prelude::*;
use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use test::{Bencher, black_box};

fn gen_ascending(len: usize) -> Vec<u64> {
    (0..len as u64).collect()
}

fn gen_descending(len: usize) -> Vec<u64> {
    (0..len as u64).rev().collect()
}

fn gen_random(len: usize) -> Vec<u64> {
    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    rng.gen_iter::<u64>().take(len).collect()
}

fn gen_mostly_ascending(len: usize) -> Vec<u64> {
    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    let mut v = gen_ascending(len);
    for _ in (0usize..).take_while(|x| x * x <= len) {
        let x = rng.gen::<usize>() % len;
        let y = rng.gen::<usize>() % len;
        v.swap(x, y);
    }
    v
}

fn gen_mostly_descending(len: usize) -> Vec<u64> {
    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    let mut v = gen_descending(len);
    for _ in (0usize..).take_while(|x| x * x <= len) {
        let x = rng.gen::<usize>() % len;
        let y = rng.gen::<usize>() % len;
        v.swap(x, y);
    }
    v
}

fn gen_strings(len: usize) -> Vec<String> {
    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    let mut v = vec![];
    for _ in 0..len {
        let n = rng.gen::<usize>() % 20 + 1;
        v.push(rng.gen_ascii_chars().take(n).collect());
    }
    v
}

fn gen_big_random(len: usize) -> Vec<[u64; 16]> {
    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    rng.gen_iter().map(|x| [x; 16]).take(len).collect()
}

macro_rules! sort {
    ($f:ident, $name:ident, $gen:expr, $len:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let v = $gen($len);
            b.iter(|| v.clone().$f());
            b.bytes = $len * mem::size_of_val(&$gen(1)[0]) as u64;
        }
    }
}

macro_rules! sort_strings {
    ($f:ident, $name:ident, $gen:expr, $len:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let v = $gen($len);
            let v = v.iter().map(|s| &**s).collect::<Vec<&str>>();
            b.iter(|| v.clone().$f());
            b.bytes = $len * mem::size_of::<&str>() as u64;
        }
    }
}

macro_rules! sort_expensive {
    ($f:ident, $name:ident, $gen:expr, $len:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let v = $gen($len);
            b.iter(|| {
                let count = AtomicUsize::new(0);
                let mut v = v.clone();

                v.$f(|a: &u64, b: &u64| {
                    // This is an intentionally expensive comparison function: we have atomic
                    // operations, landing pads due to a potential panic, a modulo operation, and
                    // trigonometric functions.
                    count.fetch_add(1, SeqCst);
                    if count.load(SeqCst) % 1_000_000_000 == 0 {
                        panic!("should not happen");
                    }
                    (*a as f64).cos().partial_cmp(&(*b as f64).cos()).unwrap()
                });

                black_box(count);
            });
            b.bytes = $len * mem::size_of_val(&$gen(1)[0]) as u64;
        }
    }
}

sort!(par_sort, par_sort_ascending, gen_ascending, 50_000);
sort!(par_sort, par_sort_descending, gen_descending, 50_000);
sort!(par_sort, par_sort_mostly_ascending, gen_mostly_ascending, 50_000);
sort!(par_sort, par_sort_mostly_descending, gen_mostly_descending, 50_000);
sort!(par_sort, par_sort_random, gen_random, 50_000);
sort!(par_sort, par_sort_big, gen_big_random, 50_000);
sort_strings!(par_sort, par_sort_strings, gen_strings, 50_000);
sort_expensive!(par_sort_by, par_sort_expensive, gen_random, 50_000);

sort!(par_sort_unstable, par_sort_unstable_ascending, gen_ascending, 50_000);
sort!(par_sort_unstable, par_sort_unstable_descending, gen_descending, 50_000);
sort!(par_sort_unstable, par_sort_unstable_mostly_ascending, gen_mostly_ascending, 50_000);
sort!(par_sort_unstable, par_sort_unstable_mostly_descending, gen_mostly_descending, 50_000);
sort!(par_sort_unstable, par_sort_unstable_random, gen_random, 50_000);
sort!(par_sort_unstable, par_sort_unstable_big, gen_big_random, 50_000);
sort_strings!(par_sort_unstable, par_sort_unstable_strings, gen_strings, 50_000);
sort_expensive!(par_sort_unstable_by, par_sort_unstable_expensive, gen_random, 50_000);

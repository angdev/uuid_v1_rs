#![feature(test)]

extern crate test;
extern crate uuid_v1;

use test::Bencher;
use uuid_v1::new_v1;

#[bench]
fn bench_new_v1(b: &mut Bencher) {
    b.iter(|| new_v1());
}
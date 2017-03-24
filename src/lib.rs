#![feature(test)]

extern crate byteorder;
extern crate chrono;
extern crate interfaces;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate test;
extern crate uuid;

use std::default::Default;
use std::sync::Mutex;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::prelude::*;
use interfaces::Interface;
use rand::Rng;

const EPOCH_START: u64 = 122192928000000000;

lazy_static! {
    static ref UUID_STORAGE: Mutex<UuidStorage> = Mutex::new(UuidStorage::default());
}

#[derive(Debug, Clone)]
pub struct Uuid([u8; 16]);

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidLength(usize),
}

impl Uuid {
    pub fn from_bytes(bytes: &[u8]) -> Result<Uuid, ParseError> {
        let len = bytes.len();
        if len != 16 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut uuid = Uuid([0; 16]);
        copy_memory(&mut uuid.0, bytes);

        Ok(uuid)
    }

    pub fn to_string(&self) -> String {
        format!("{:08x}-{:04x}-{:04x}-{:04x}-{:04x}{:08x}",
                (&self.0[0..4]).read_u32::<BigEndian>().unwrap(),
                (&self.0[4..6]).read_u16::<BigEndian>().unwrap(),
                (&self.0[6..8]).read_u16::<BigEndian>().unwrap(),
                (&self.0[8..10]).read_u16::<BigEndian>().unwrap(),
                (&self.0[10..12]).read_u16::<BigEndian>().unwrap(),
                (&self.0[12..16]).read_u32::<BigEndian>().unwrap())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &(self.0)
    }

    fn set_version(&mut self, v: u8) {
        self.0[6] = (self.0[6] & 0xF) | ((v as u8) << 4);
    }

    fn set_variant(&mut self) {
        self.0[8] = (self.0[8] & 0x3f) | 0x80;
    }
}

#[derive(Debug)]
struct UuidStorage {
    clock_seq: u16,
    hardware_addr: [u8; 6],
    last_time: u64,
}

#[derive(Debug)]
struct UuidStorageState(u64, u16, [u8; 6]);

impl UuidStorage {
    fn new() -> UuidStorage {
        let clock_seq = init_clock_seq();
        let mut buffer = [0u8; 6];
        init_hardware_addr(&mut buffer);

        UuidStorage {
            clock_seq: clock_seq,
            hardware_addr: buffer,
            last_time: 0,
        }
    }

    fn get_state(&mut self) -> UuidStorageState {
        let time_now = unix_time_func();
        if time_now <= self.last_time {
            self.clock_seq += 1;
        }
        self.last_time = time_now;

        UuidStorageState(time_now, self.clock_seq, self.hardware_addr.clone())
    }
}

impl Default for UuidStorage {
    fn default() -> UuidStorage {
        UuidStorage::new()
    }
}

fn init_clock_seq() -> u16 {
    let mut buffer = [0u8; 2];
    rand::thread_rng().fill_bytes(&mut buffer);
    (&buffer[..]).read_u16::<BigEndian>().unwrap()
}

fn init_hardware_addr(mut buffer: &mut [u8]) {
    use std::u8;

    let result = match Interface::get_all() {
        Ok(interfaces) => interfaces.first().and_then(|i| i.hardware_addr().ok()),
        Err(_) => None,
    };

    match result {
        Some(addr) => {
            let addr_bytes: Vec<u8> = addr.as_string()
                .split(":")
                .map(|hex| u8::from_str_radix(hex, 16).unwrap())
                .collect();
            copy_memory(&mut buffer, addr_bytes.as_slice())
        }
        None => rand::thread_rng().fill_bytes(&mut buffer),
    }
}

fn unix_time_func() -> u64 {
    let now = UTC::now();
    let timestamp = now.timestamp() as u64;
    let nanos = now.timestamp_subsec_nanos() as u64;
    EPOCH_START + (timestamp * 1_000_000_000 + nanos) / 100
}

fn copy_memory(dst: &mut [u8], src: &[u8]) {
    for (slot, val) in dst.iter_mut().zip(src.iter()) {
        *slot = *val;
    }
}

pub fn new_v1() -> Uuid {
    let mut storage = UUID_STORAGE.lock().unwrap();
    let UuidStorageState(time_now, clock_seq, hardware_addr) = storage.get_state();

    let mut buffer = [0u8; 16];

    (&mut buffer[0..4]).write_u32::<BigEndian>(time_now as u32).unwrap();
    (&mut buffer[4..6]).write_u16::<BigEndian>((time_now >> 32) as u16).unwrap();
    (&mut buffer[6..8]).write_u16::<BigEndian>((time_now >> 48) as u16).unwrap();
    (&mut buffer[8..10]).write_u16::<BigEndian>(clock_seq as u16).unwrap();

    copy_memory(&mut buffer[10..], &hardware_addr);

    let mut uuid = Uuid::from_bytes(&buffer).unwrap();

    uuid.set_variant();
    uuid.set_version(1);

    uuid
}

#[cfg(test)]
mod tests {
    use super::new_v1;
    use test::Bencher;

    #[bench]
    fn bench_new_v1(b: &mut Bencher) {
        b.iter(|| new_v1());
    }
}

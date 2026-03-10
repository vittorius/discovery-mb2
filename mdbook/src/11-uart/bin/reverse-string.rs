#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::Vec;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // A buffer with 32 bytes of capacity
    let mut buffer: Vec<u8, 32> = Vec::new();

    loop {
        let byte = serial.read().unwrap();
        serial.write(byte).unwrap();
        if byte == b'\r' {
            serial.write_str("\r\n").unwrap();
            buffer.reverse();
            for b in buffer.iter() {
                serial.write(*b).unwrap();
            }
            buffer.clear();
            serial.write_str("\r\n").unwrap();
        } else {
            let res = buffer.push(byte);
            if res.is_err() {
                write!(serial, "\r\n[ERROR] Buffer overflow, resetting buffer. The buffer was: {}\r\n", str::from_utf8(buffer.as_slice()).unwrap()).unwrap();
                buffer.clear();
            }
        }
        serial.flush().unwrap();
        

        // TODO Receive a user request. Each user request ends with ENTER
        // NOTE `buffer.push` returns a `Result`. Handle the error by responding
        // with an error message.

        // TODO Send back the reversed string
    }
}

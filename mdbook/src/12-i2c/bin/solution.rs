#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::Vec;
use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr, interface::I2cInterface, mode::{MagContinuous, MagOneShot}};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use microbit::{
    hal::{
        Timer, twim::Twim, uarte::{self, Baudrate, Parity}
    },
    pac::{TWIM0, twim0::frequency::FREQUENCY_A},
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

    let i2c = Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
    let mut timer0 = Timer::new(board.TIMER0);

    // Code from documentation
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut timer0,
            AccelMode::HighResolution,
            AccelOutputDataRate::Hz50,
        )
        .unwrap();

    let mut sensor = sensor.into_mag_continuous().ok().unwrap();
    let mut cmd_buf: Vec<u8, 3> = Vec::new();

    loop {
        let byte = serial.read().unwrap();
        serial.write(byte).unwrap();
        if byte == b'\r' {
            let cmd = str::from_utf8(&cmd_buf).unwrap();
            write!(serial, "\r\n").unwrap();
            match cmd {
                "acc" => { print_acc_data(&mut serial, &mut sensor) }
                "mag" => { print_mag_data(&mut serial, &mut sensor) }
                _ => {
                    write!(serial, "\r\n[ERROR] Unknown command: {}\r\n", cmd).unwrap();
                }
            }
            cmd_buf.clear();
        } else {
            let res = cmd_buf.push(byte);
            if res.is_err() {
                write!(
                    serial,
                    "\r\n[ERROR] Cmd buffer overflow, resetting\r\n",
                )
                .unwrap();
                cmd_buf.clear();
            }
        }
        serial.flush().unwrap();
    }
}

fn print_mag_data(serial: &mut impl Write, sensor: &mut Lsm303agr<I2cInterface<Twim<TWIM0>>, MagContinuous>) {
    if sensor.mag_status().unwrap().xyz_new_data() {
        let (x, y, z) = sensor.magnetic_field().unwrap().xyz_raw();
        write!(serial, "Magnetic field: x {} y {} z {}\r\n", x, y, z).unwrap();
    }
}

fn print_acc_data(serial: &mut impl Write, sensor: &mut Lsm303agr<I2cInterface<Twim<TWIM0>>, MagContinuous>) {
    if sensor.accel_status().unwrap().xyz_new_data() {
        let (x, y, z) = sensor.acceleration().unwrap().xyz_mg();
        write!(serial, "Acceleration: x {} y {} z {}\r\n", x, y, z).unwrap();
    }
}

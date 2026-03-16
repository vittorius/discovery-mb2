#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    display::blocking::Display,
    hal::{twim, Timer},
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr};

// const R: f32 = 2.5;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut timer0 = Timer::new(board.TIMER0);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut timer0,
            AccelMode::HighResolution,
            AccelOutputDataRate::Hz10,
        )
        .unwrap();

    let mut x_g: Option<f32> = None;
    loop {
        while !sensor.accel_status().unwrap().xyz_new_data() {
            nop();
        }
        let (x, _, _) = sensor.acceleration().unwrap().xyz_mg();
        let x = x as f32 / 1000.0;

        if x > 1.0 {
            x_g = Some(x);
        } else {
            let prev_x_g = x_g.take();
            if let Some(prev_x_g) = prev_x_g {
                rprintln!("Punch X was {}g", prev_x_g);
            }
        }
    }
}

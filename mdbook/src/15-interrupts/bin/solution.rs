#![no_main]
#![no_std]

use cortex_m::asm::wfi;
use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use libm::floor;
use microbit::{
    hal::{gpio::Level, Timer},
    Board,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut speaker_pin = board.speaker_pin.into_push_pull_output(Level::Low);
    let mut timer = Timer::new(board.TIMER0);

    for _ in 0..10 {
        for freq in [440, 660].iter() {
            rprintln!("freq: {}", freq);

            let period = floor(1_000_000.0 / (*freq as f64)) as u32;
            rprintln!("period: {}", period);

            for _ in 0..(250_000 / period) {
                speaker_pin.set_high().unwrap();
                timer.delay_us(period / 2);
                speaker_pin.set_low().unwrap();
                timer.delay_us(period / 2);
            }
        }
    }

    loop {
        wfi();
    }
}

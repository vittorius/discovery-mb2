#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use microbit::display::blocking::Display;
use microbit::hal::timer::Timer;
use microbit::{hal::gpio, Board};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

const LEFT_TURN: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 1, 0, 0, 0],
    [1, 1, 1, 1, 1],
    [0, 1, 0, 0, 0],
    [0, 0, 1, 0, 0],
];
const RIGHT_TURN: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 0, 0, 1, 0],
    [1, 1, 1, 1, 1],
    [0, 0, 0, 1, 0],
    [0, 0, 1, 0, 0],
];
const DEFAULT: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    // Configure buttons
    let mut button_a = board.buttons.button_a;
    let mut button_b = board.buttons.button_b;

    // Configure display
    let mut display = Display::new(board.display_pins);

    // loop {
    //     let left_pressed = button_a.is_low().unwrap();
    //     let right_pressed = button_b.is_low().unwrap();
    //     match (left_pressed, right_pressed) {
    //         // Stay in current state until something is pressed.
    //         (false, false) => display.show(&mut timer, DEFAULT, 10),
    //         // Change to on state.
    //         (true, false) => display.show(&mut timer, LEFT_TURN, 10),
    //         // Change to off state.
    //         (false, true) => display.show(&mut timer, RIGHT_TURN, 10),
    //         // Stay in current state until something is released.
    //         (true, true) => (),
    //     }
    //     // timer.delay_ms(10_u32);
    loop {
           if button_a.is_low().unwrap() {
               // Blink left arrow
               display.show(&mut timer, LEFT_TURN, 500);
               timer.delay_ms(500_u32);
               display.clear();
               timer.delay_ms(500_u32);
           } else if button_b.is_low().unwrap() {
               // Blink right arrow
               display.show(&mut timer, RIGHT_TURN, 500);
               timer.delay_ms(500_u32);
               display.clear();
               timer.delay_ms(500_u32);
           } else {
               display.show(&mut timer, DEFAULT, 500);
           }
           timer.delay_ms(10_u32);
       }   // }
}

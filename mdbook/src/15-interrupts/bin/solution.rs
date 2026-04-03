#![no_main]
#![no_std]

use cortex_m::asm::wfi;
use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use embedded_hal::digital::StatefulOutputPin;
use microbit::{
    hal::{
        gpio::{p0::P0_00, Disconnected, Level, Output, PushPull},
        Timer,
    },
    pac::{self, interrupt, NVIC, TIMER0, TIMER1},
    Board,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[derive(Clone, Copy)]
#[repr(u32)]
enum Frequency {
    Hz440 = 440,
    Hz660 = 660,
}

struct Siren {
    freq: Frequency,
    half_period_cycles_left: u32,
    period_us: u32,
    speaker_pin: P0_00<Output<PushPull>>,
    timer: Timer<TIMER0>,
    running: bool,
}

impl Siren {
    fn new(speaker_pin: P0_00<Disconnected>, timer: TIMER0) -> Self {
        let speaker_pin = speaker_pin.into_push_pull_output(Level::Low);
        let timer = Timer::new(timer);

        let freq = Frequency::Hz440;

        Self {
            freq,
            half_period_cycles_left: Self::half_period_cycles_from(freq),
            period_us: Self::period_us(freq),
            speaker_pin,
            timer,
            running: false,
        }
    }

    fn start(&mut self) {
        self.timer.enable_interrupt();
        unsafe { NVIC::unmask(pac::Interrupt::TIMER0) };
        NVIC::unpend(pac::Interrupt::TIMER0);

        self.running = true;

        self.timer.start(self.period_us / 2); // timer fires each 1/2 of a sound frequency period
    }

    fn update(&mut self) {
        if !self.running {
            return;
        }

        if self.half_period_cycles_left == 0 {
            self.freq = match self.freq {
                Frequency::Hz440 => Frequency::Hz660,
                Frequency::Hz660 => Frequency::Hz440,
            };
            self.half_period_cycles_left = Self::half_period_cycles_from(self.freq);
            self.period_us = Self::period_us(self.freq);
        } else {
            self.half_period_cycles_left -= 1;
            self.speaker_pin.toggle();
            self.timer.start(self.period_us / 2); // timer fires each 1/2 of a sound frequency period
        }
    }

    fn stop(&mut self) {
        self.running = false;
    }

    fn half_period_cycles_from(freq: Frequency) -> u32 {
        // Siren must blare for 1/4 of a second on a given frequency; for precision, time unit is us (microsecond).
        // We take half-periods as cycle units because we must update the state machine state every 1/2 of a siren period.
        2 * 250_000 / Self::period_us(freq)
    }

    fn period_us(freq: Frequency) -> u32 {
        1_000_000 / freq as u32
    }
}

#[interrupt]
fn TIMER0() {
    SIREN.with_lock(|siren| {
        siren.update();
    });
}

struct Launcher {
    ticks_left: u8,
    timer: Timer<TIMER1>,
}

impl Launcher {
    fn new(timer: TIMER1) -> Self {
        Self {
            ticks_left: 10,
            timer: Timer::new(timer),
        }
    }

    fn start(&mut self) {
        self.timer.enable_interrupt();
        unsafe { NVIC::unmask(pac::Interrupt::TIMER1) };
        NVIC::unpend(pac::Interrupt::TIMER1);

        self.timer.start(1_000_000); // 1 sec
    }

    fn stop(&mut self) {
        NVIC::mask(pac::Interrupt::TIMER1);
    }

    fn count_down(&mut self) {
        if self.ticks_left > 0 {
            self.ticks_left -= 1;
            self.timer.start(1_000_000); // 1 sec
        } else {
            self.stop();
            SIREN.with_lock(|siren| siren.stop());

            rprintln!("launch");
        }
    }
}

#[interrupt]
fn TIMER1() {
    LAUNCHER.with_lock(|launcher| {
        rprintln!("{}", launcher.ticks_left);
        launcher.count_down();
    });
}

static SIREN: LockMut<Siren> = LockMut::new();
static LAUNCHER: LockMut<Launcher> = LockMut::new();

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();

    SIREN.init(Siren::new(board.speaker_pin, board.TIMER0));
    SIREN.with_lock(|siren| siren.start());

    LAUNCHER.init(Launcher::new(board.TIMER1));
    LAUNCHER.with_lock(|launcher| launcher.start());

    loop {
        wfi();
    }
}

#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting;
extern crate embedded_graphics;
extern crate panic_semihosting;
extern crate stm32f1xx_hal as hal;

use nb::block;

use cast::{u16, u32};

use cortex_m_rt::entry;

use hal::i2c::{BlockingI2c, DutyCycle, Mode};

use ssd1306::{prelude::*, Builder};

use stm32f1xx_hal::{pac, pac::TIM2, prelude::*, rcc::Clocks, time::Hertz, timer::Timer};

fn set_pwn_freq(clocks: Clocks, freq: Hertz) {
    unsafe {
        let clk = clocks.pclk1_tim().0;
        let freq = freq.0;
        let ticks = clk / freq;
        let psc = u16(ticks / (1 << 16)).unwrap();
        (*TIM2::ptr()).psc.write(|w| w.psc().bits(psc));
        let arr = u16(ticks / u32(psc + 1)).unwrap();
        (*TIM2::ptr()).arr.write(|w| w.arr().bits(arr));

        (*TIM2::ptr()).cr1.write(|w| {
            w.cms()
                .bits(0b00)
                .dir()
                .clear_bit()
                .opm()
                .clear_bit()
                .cen()
                .set_bit()
        });
    }
}

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, 20.hz(), clocks);

    /* init display ---------------------------------------------------------------------- */
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // Acquire the GPIOB peripheral

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    /* main loop --------------------------------------------------------------*/

    let mut gpio_a = dp.GPIOA.split(&mut rcc.apb2);
    let pa0 = gpio_a.pa0.into_alternate_push_pull(&mut gpio_a.crl);
    let mut pwm = dp
        .TIM2
        .pwm(pa0, &mut afio.mapr, 440.hz(), clocks, &mut rcc.apb1);

    pwm.set_duty(pwm.get_max_duty() / 2);

    // Wait for the timer to trigger an update and change the state of the LED

    let mut i = 1u32;
    let mut m = 1u32;
    let mut n = 1u32;
    loop {
        set_pwn_freq(clocks, (1 + i * m).hz());
        pwm.enable();

        set_pwn_freq(clocks, (1 + i * m % 500).hz());
        block!(timer.wait()).unwrap();
        pwm.disable();
        led.set_high();

        block!(timer.wait()).unwrap();

        i += 1;
        n += 1;
        if n > m {
            m = n;
            n = 1;
        }

        match i % 4 {
            0 => {
                disp.set_rotation(DisplayRotation::Rotate0).unwrap();
                disp.set_pixel(m % 80, m % n, 1);
            }
            1 => {
                disp.set_rotation(DisplayRotation::Rotate90).unwrap();
                disp.set_pixel(n % m, (m - i) % 87, 0);
            }
            2 => {
                disp.set_rotation(DisplayRotation::Rotate180).unwrap();
                disp.set_pixel(i, n * m % 201, 1);
            }
            3 => {
                disp.set_rotation(DisplayRotation::Rotate270).unwrap();
                disp.set_pixel((m - i) % 207, n, 1);
            }
            _ => {}
        };
        disp.flush().unwrap();
        led.set_low();

        i += 1;
    }
}

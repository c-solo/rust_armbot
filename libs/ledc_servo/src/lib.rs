//! This is a small lib for controlling servo using LEDC.
//! Api the same as stepper motors with 2 methods: `step` and `dir`.

#![allow(unused)]

use std::{marker::PhantomData, ops::Range, time::Duration};

use esp_idf_hal::{
    gpio::OutputPin,
    ledc,
    ledc::{LedcChannel, LedcTimer, LowSpeed, SpeedMode},
    peripheral::Peripheral,
    prelude::{FromValueType, Hertz},
};
use esp_idf_sys::EspError;
use log::{info, trace};

#[derive(Debug, Clone)]
pub struct ServoConfig {
    /// Max angle that servo can't be turned, mostly 180, 360.
    pub max_angle: f64,
    /// What frequency expect servo (ex. 50Hz for SG90).
    pub frequency: Hertz,
    /// What pulse width servo supports (ex. 500-2400ns for SG90).
    pub pulse_width_ns: Range<u32>,
    /// PWM resolution in bits.
    pub resolution: ledc::Resolution,
    /// How much add\subtract to 'duty' for making micro step
    pub step: u32,
}

impl ServoConfig {
    /// Config for [SG90](https://www.friendlywire.com/projects/ne555-servo-safe/SG90-datasheet.pdf).
    pub fn sg90() -> Self {
        let pulse_width_ns = 500..2600;
        let max_angle = 180.0;
        let step = 5;
        ServoConfig {
            max_angle,
            frequency: Hertz(50),
            pulse_width_ns,
            resolution: ledc::Resolution::Bits12,
            step,
        }
    }

    /// The same config as for [SG90](https://www.friendlywire.com/projects/ne555-servo-safe/SG90-datasheet.pdf).
    fn sg90s() -> Self {
        Self::sg90()
    }
}

pub struct Servo<'d> {
    name: String,
    pub ledc_driver: ledc::LedcDriver<'d>,
    pub duty: Range<u32>,
    config: ServoConfig,
    /// Current direction. True - forward, false - backward.
    direction: bool,
    _p: PhantomData<&'d mut ()>,
}

impl<'d> Servo<'d> {
    /// Creates new servo driver instance for LowSpeed LEDC channel because
    /// ESP32S2, ESP32S3, ESP32C2 and ESP32C3 supports Low Speed Mode.
    pub fn new<T, C, P>(
        name: &str,
        config: ServoConfig,
        timer: impl Peripheral<P = T> + 'd,
        channel: impl Peripheral<P = C> + 'd,
        pin: impl Peripheral<P = P> + 'd,
    ) -> Result<Servo<'d>, EspError>
    where
        T: LedcTimer<SpeedMode = LowSpeed> + 'd,
        C: LedcChannel<SpeedMode = LowSpeed> + 'd,
        P: OutputPin,
    {
        let timer_config = ledc::config::TimerConfig::default()
            .resolution(config.resolution)
            .frequency(config.frequency);

        let timer_driver = ledc::LedcTimerDriver::new(timer, &timer_config)?;

        let mut ledc_driver = ledc::LedcDriver::new(channel, timer_driver, pin)?;

        let duty_range = calc_duty_range(&config, ledc_driver.get_max_duty());

        // set to center position
        let center = duty_range.start + (duty_range.end - duty_range.start) / 2;
        ledc_driver.set_duty(center);

        info!(
            "{name} servo: center={center}, duty_range={duty_range:?}",
            name = name,
            center = center,
            duty_range = duty_range
        );

        Ok(Servo {
            name: name.to_string(),
            ledc_driver,
            duty: duty_range,
            config,
            direction: true,
            _p: PhantomData,
        })
    }

    /// Make micro step, return false if servo reaches min or max position.
    pub fn step(&mut self, step: u32) -> Result<bool, EspError> {
        let max_duty = self.ledc_driver.get_max_duty();
        let new_duty = self.calc_duty(step);

        if new_duty > self.duty.end || new_duty < self.duty.start {
            // servo reaches bounds, skip step
            return Ok(false);
        }

        self.ledc_driver.set_duty(new_duty);
        self.ledc_driver.enable();
        trace!("{} servo step({}) to {}", &self.name, step, new_duty);
        Ok(true)
    }

    /// Set servo to move forward (increase angle).
    pub fn forward(&mut self) {
        self.direction = true;
    }

    /// Set servo to move backward (decrease angle).
    pub fn backward(&mut self) {
        self.direction = false;
    }

    /// Returns true if servo is set to move forward (increase angle).
    pub fn is_forward(&self) -> bool {
        self.direction
    }

    /// Sets new direction value, returns old direction value.
    /// Prefer using `forward()` and `backward()` methods instead.
    #[deprecated(since = "0.2.0", note = "Use `forward()` or `backward()` instead")]
    pub fn dir(&mut self, direction: bool) -> bool {
        let old = self.direction;
        self.direction = direction;
        old
    }

    /// Returns current angle value.
    pub fn get_angle(&self) -> f64 {
        let max_duty = self.ledc_driver.get_max_duty();
        let current_duty = self.ledc_driver.get_duty();
        calculate_angle(&self.config, current_duty, max_duty)
    }

    fn calc_duty(&self, mut step: u32) -> u32 {
        let current_duty = self.ledc_driver.get_duty();
        if self.direction {
            current_duty + step
        } else {
            current_duty.max(step) - step
        }
    }
}

const NANOS_IS_SEC: f64 = 1_000_000.0;

fn calc_duty_range(config: &ServoConfig, max_duty: u32) -> Range<u32> {
    let min_pulse = config.pulse_width_ns.start;
    let max_pulse = config.pulse_width_ns.end;
    let min_duty = pulse_to_duty(config, min_pulse, max_duty);
    let max_duty = pulse_to_duty(config, max_pulse, max_duty);
    min_duty..max_duty
}


/// Transforms 'duty' to 'angle' in respect that given servo pulse range.
fn calculate_angle(config: &ServoConfig, duty: u32, max_duty: u32) -> f64 {
    let pulse_ns = (duty as f64 * NANOS_IS_SEC / config.frequency.0 as f64 / max_duty as f64);

    (pulse_ns - config.pulse_width_ns.start as f64)
        / (config.pulse_width_ns.end - config.pulse_width_ns.start) as f64
        * config.max_angle
}

fn pulse_to_duty(config: &ServoConfig, pulse: u32, max_duty: u32) -> u32 {
    let duty = pulse as f64 * config.frequency.0 as f64 * max_duty as f64 / NANOS_IS_SEC;
    duty as u32
}

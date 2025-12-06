#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    ledc::{channel, timer, timer::config::Duty, Ledc},
    peripherals::{ADC1, GPIO0, GPIO1, GPIO2, GPIO3},
    Config,
};
use esp_hal_servo::{Servo, ServoConfig};

use crate::{
    armbot::{ArmBot, ArmBotConfig},
    gamepad::{GamepadConfig, GamepadImpl},
};

mod armbot;
mod error;
mod gamepad;
mod util;

esp_bootloader_esp_idf::esp_app_desc!();

#[riscv_rt::entry]
fn main() -> ! {
    let peripherals = esp_hal::init(Config::default());

    let servo_cfg = ServoConfig::sg90(Duty::Duty14Bit);
    let mut ledc = Ledc::new(peripherals.LEDC);
    let timer = servo_cfg
        .configure_timer(
            &mut ledc,
            timer::Number::Timer0,
            timer::LSClockSource::APBClk,
        )
        .expect("failed to configure timer");

    let shoulder_servo = Servo::new(
        "shoulder",
        servo_cfg.clone(),
        &mut ledc,
        &timer,
        channel::Number::Channel0,
        peripherals.GPIO5,
    )
    .expect("shoulder init failed");

    let elbow_servo = Servo::new(
        "elbow",
        servo_cfg.clone(),
        &mut ledc,
        &timer,
        channel::Number::Channel0,
        peripherals.GPIO6,
    )
    .expect("elbow init failed");

    let gripper_servo = Servo::new(
        "gripper",
        servo_cfg,
        &mut ledc,
        &timer,
        channel::Number::Channel0,
        peripherals.GPIO7,
    )
    .expect("gripper init failed");

    let gamepad: GamepadImpl<ADC1, GPIO0, GPIO1, GPIO2, GPIO3> = GamepadImpl::new(
        GamepadConfig {
            center_offset: 100,
            ..GamepadConfig::default()
        },
        peripherals.ADC1,
        peripherals.GPIO0,
        peripherals.GPIO1,
        peripherals.GPIO2,
        peripherals.GPIO3,
    )
    .expect("gamepad init failed");

    let mut bot = ArmBot::new(
        ArmBotConfig::default(),
        gamepad,
        shoulder_servo,
        elbow_servo,
        gripper_servo,
    )
    .expect("ArmBot init failed");

    log::info!("Arm bot initialized");

    let delay = Delay::new();
    loop {
        if let Err(e) = bot.do_step() {
            log::error!("step failed: {:?}", e);
        }
        delay.delay_millis(10); // todo remove
    }
}

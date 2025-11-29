use std::ops::Range;

use ledc_servo::Servo;

use crate::gamepad::{Gamepad, Position};

#[allow(unused)] // todo remove allow
pub struct ArmBot<'d, G> {
    config: ArmBotConfig,

    // pub base: Motor,
    shoulder_servo: Servo<'d>,
    elbow_servo: Servo<'d>,
    gripper_servo: Servo<'d>,

    gamepad: G,
    elbow_angle: f64,
    shoulder_angle: f64,
    gripper_angle: f64,
}

impl<'d, G: Gamepad> ArmBot<'d, G> {
    pub fn new(
        config: ArmBotConfig,
        gamepad: G,
        shoulder_servo: Servo<'d>,
        elbow_servo: Servo<'d>,
        gripper_servo: Servo<'d>,
    ) -> eyre::Result<Self> {
        Ok(Self {
            config,

            shoulder_servo,
            elbow_servo,
            gripper_servo,

            gamepad,
            elbow_angle: 0.0,
            shoulder_angle: 0.0,
            gripper_angle: 0.0,
        })
    }

    /// Makes the arm bot do a cycle of its movement.
    pub fn do_step(&mut self) -> eyre::Result<()> {
        let state = self.gamepad.read_state(&self.config.step_size)?;
        if state.is_center() {
            // noting to do
            return Ok(());
        }

        Self::make_step(&state.shoulder, &mut self.shoulder_servo)?;
        Self::make_step(&state.elbow, &mut self.elbow_servo)?;
        Self::make_step(&state.gripper, &mut self.gripper_servo)?;
        // todo add base_rotator

        Ok(())
    }

    pub fn make_step(cmd: &Position, servo: &mut Servo<'d>) -> eyre::Result<()> {
        match cmd {
            Position::Center => {
                // do nothing
            }
            Position::Low(step) => {
                servo.forward();
                servo.forward();
                servo.step(*step)?;
            }
            Position::High(step) => {
                servo.backward();
                servo.step(*step)?;
            }
        }
        Ok(())
    }
}

#[allow(unused)] // todo remove allow
pub struct ArmBotConfig {
    /// Desirable range of the shoulder angle.
    pub shoulder_angle_range: Range<usize>,
    /// Desirable range of the elbow angle.
    pub elbow_angle_range: Range<usize>,
    /// Desirable range of the gripper angle.
    pub gripper_angle_range: Range<usize>,

    /// Min possible step, for slowest motion.
    /// Max possible step, for fastest motion.
    pub step_size: Range<u32>,
}

impl Default for ArmBotConfig {
    fn default() -> Self {
        Self {
            shoulder_angle_range: 30..150,
            elbow_angle_range: 30..150,
            gripper_angle_range: 20..70,
            step_size: 1..10,
        }
    }
}

use core::ops::Range;

use esp_hal::{
    analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, Attenuation, RegisterAccess},
    gpio::AnalogPin,
    Blocking,
};
use log::{debug, info};

use crate::{error::Error, util};

pub struct GamepadConfig {
    /// Min value of joystick.
    pub joystick_min_value: u32,
    /// Max value of joystick.
    pub joystick_max_value: u32,

    /// Defines center of joystick as offsets from middle point.
    /// `min value = center - center_offset`
    /// `max value = center + center_offset`
    pub center_offset: u32,

    /// If set to true, then real center position will be read from the joystick at the start.
    pub use_real_center: bool,
}

impl GamepadConfig {
    /// Sets offset `[center-offset, center+offset]` that will be considered as center.
    fn center_range(&self, offset: u32) -> Range<u32> {
        offset - self.center_offset..offset + self.center_offset
    }
}

impl Default for GamepadConfig {
    fn default() -> Self {
        Self {
            joystick_min_value: 10,
            joystick_max_value: 2757,
            center_offset: 50,
            use_real_center: true,
        }
    }
}

pub trait Gamepad {
    /// Returns raw values of joystick.
    fn read_raw_state(&mut self) -> Result<RawState, Error>;

    /// Returns state of joystick mapped to the specified output range.
    fn read_state(&mut self, output: &Range<u32>) -> Result<State, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct RawState {
    pub base_rotator: u32,
    pub shoulder: u32,
    pub elbow: u32,
    pub gripper: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    pub base_rotator: Position,
    pub shoulder: Position,
    pub elbow: Position,
    pub gripper: Position,
}

impl State {
    pub fn is_center(&self) -> bool {
        self.base_rotator == Position::Center
            && self.shoulder == Position::Center
            && self.elbow == Position::Center
            && self.gripper == Position::Center
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Position {
    Low(u32),
    #[default]
    Center,
    High(u32),
}

impl Position {
    fn new(
        val: u32,
        config: &GamepadConfig,
        center_range: &Range<u32>,
        output: &Range<u32>,
    ) -> Self {
        if center_range.contains(&val) {
            Position::Center
        } else if val < center_range.start {
            let val = util::map(
                val,
                config.joystick_min_value,
                center_range.start,
                output.start,
                output.end,
                true,
            );
            Position::Low(val)
        } else {
            let val = util::map(
                val,
                center_range.end,
                config.joystick_max_value,
                output.start,
                output.end,
                false,
            );
            Position::High(val)
        }
    }
}

pub struct GamepadImpl<'d, ADC: RegisterAccess + 'd, P0, P1, P2, P3> {
    config: GamepadConfig,

    /// ADC driver in blocking mode.
    adc: Adc<'d, ADC, Blocking>,
    base_rotator_pin: AdcPin<P0, ADC>,
    shoulder_pin: AdcPin<P1, ADC>,
    elbow_pin: AdcPin<P2, ADC>,
    gripper_pin: AdcPin<P3, ADC>,

    base_rotator_center: Range<u32>,
    shoulder_center: Range<u32>,
    elbow_center: Range<u32>,
    gripper_center: Range<u32>,
}

impl<'d, ADC, P0, P1, P2, P3> GamepadImpl<'d, ADC, P0, P1, P2, P3>
where
    ADC: RegisterAccess + 'd,
    P0: AnalogPin + AdcChannel,
    P1: AnalogPin + AdcChannel,
    P2: AnalogPin + AdcChannel,
    P3: AnalogPin + AdcChannel,
{
    pub fn new(
        config: GamepadConfig,
        adc: ADC,
        base_rotator_pin: P0,
        shoulder_pin: P1,
        elbow_pin: P2,
        gripper_pin: P3,
    ) -> Result<Self, Error> {
        let mut adc_config = AdcConfig::new();
        let base_rotator_pin = adc_config.enable_pin(base_rotator_pin, Attenuation::_11dB);
        let shoulder_pin = adc_config.enable_pin(shoulder_pin, Attenuation::_11dB);
        let elbow_pin = adc_config.enable_pin(elbow_pin, Attenuation::_11dB);
        let gripper_pin = adc_config.enable_pin(gripper_pin, Attenuation::_11dB);
        let adc = Adc::new(adc, adc_config);

        let default_center_range = config.center_range(config.joystick_max_value / 2);
        let mut gamepad = Self {
            config,
            adc,
            base_rotator_pin,
            shoulder_pin,
            elbow_pin,
            gripper_pin,
            base_rotator_center: default_center_range.clone(),
            shoulder_center: default_center_range.clone(),
            elbow_center: default_center_range.clone(),
            gripper_center: default_center_range.clone(),
        };

        if gamepad.config.use_real_center {
            // read and store center position
            let real_positions = gamepad.read_raw_state()?;
            gamepad.base_rotator_center = gamepad.config.center_range(real_positions.base_rotator);
            gamepad.shoulder_center = gamepad.config.center_range(real_positions.shoulder);
            gamepad.elbow_center = gamepad.config.center_range(real_positions.elbow);
            gamepad.gripper_center = gamepad.config.center_range(real_positions.gripper);
        }
        info!("base_rotator center={:?}", gamepad.base_rotator_center);
        info!("shoulder center={:?}", gamepad.shoulder_center);
        info!("elbow center={:?}", gamepad.elbow_center);
        info!("gripper center={:?}", gamepad.gripper_center);

        Ok(gamepad)
    }
}

impl<'d, ADC, P0, P1, P2, P3> Gamepad for GamepadImpl<'d, ADC, P0, P1, P2, P3>
where
    ADC: RegisterAccess + 'd,
    P0: AnalogPin + AdcChannel,
    P1: AnalogPin + AdcChannel,
    P2: AnalogPin + AdcChannel,
    P3: AnalogPin + AdcChannel,
{
    fn read_raw_state(&mut self) -> Result<RawState, Error> {
        let base_rotator_angle = self
            .adc
            .read_oneshot(&mut self.base_rotator_pin)
            .map_err(|_| Error::Adc)? as u32;
        let shoulder_angle = self
            .adc
            .read_oneshot(&mut self.shoulder_pin)
            .map_err(|_| Error::Adc)? as u32;
        let elbow_angle = self
            .adc
            .read_oneshot(&mut self.elbow_pin)
            .map_err(|_| Error::Adc)? as u32;
        let gripper_angle = self
            .adc
            .read_oneshot(&mut self.gripper_pin)
            .map_err(|_| Error::Adc)? as u32;

        fn normalize_value(val: u32, config: &GamepadConfig) -> u32 {
            val.min(config.joystick_max_value)
                .max(config.joystick_min_value)
        }

        let state = RawState {
            base_rotator: normalize_value(base_rotator_angle, &self.config),
            shoulder: normalize_value(shoulder_angle, &self.config),
            elbow: normalize_value(elbow_angle, &self.config),
            gripper: normalize_value(gripper_angle, &self.config),
        };
        debug!("raw state = {:?}", state);
        Ok(state)
    }

    fn read_state(&mut self, output: &Range<u32>) -> Result<State, Error> {
        let state = self.read_raw_state()?;
        let state = State {
            base_rotator: Position::new(
                state.base_rotator,
                &self.config,
                &self.base_rotator_center,
                output,
            ),
            shoulder: Position::new(state.shoulder, &self.config, &self.shoulder_center, output),
            elbow: Position::new(state.elbow, &self.config, &self.elbow_center, output),
            gripper: Position::new(state.gripper, &self.config, &self.gripper_center, output),
        };
        debug!("state = {:?}", state);
        Ok(state)
    }
}

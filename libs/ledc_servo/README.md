# ledc_servo

A small library for controlling servo motors using ESP32 LEDC (LED Control) peripheral.

## Features

- Control servo motors using PWM via LEDC
- Support for custom servo configurations (SG90/SG90S preconfigured)
- Angle calculation and position tracking

## Usage

```rust
use ledc_servo::{Servo, ServoConfig};
use esp_idf_hal::peripherals::Peripherals;

let peripherals = Peripherals::take().unwrap();

// Use pre-configured SG90 settings
let servo_cfg = ServoConfig::sg90();

// Create servo instance
let mut servo = Servo::new(
    "my_servo",
    servo_cfg,
    peripherals.ledc.timer0,
    peripherals.ledc.channel0,
    peripherals.pins.gpio5,
)?;

// Control servo
servo.forward();  // Set direction to forward (increase angle)
servo.step(5)?;   // Move by 5 steps

servo.backward(); // Set direction to backward (decrease angle)
servo.step(3)?;   // Move by 3 steps

// Get current angle and direction
let angle = servo.get_angle();
let is_forward = servo.is_forward();
```

## API Methods

### Direction Control
- `servo.forward()` - Set servo to move forward (increase angle)
- `servo.backward()` - Set servo to move backward (decrease angle)
- `servo.is_forward()` - Returns true if servo is set to move forward
- `servo.step(n)` - Move servo by `n` steps in current direction. Returns `Ok(false)` if servo reaches bounds.
- `servo.get_angle()` - Get current angle in degrees
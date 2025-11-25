## Rust firmware for robotic arm based on ESP32-C3 SuperMini board.

Electronic parts:

- Esp32-C3 SuperMini (any ESP32 C3 or C6 board is suitable).
- 2 joystick modules (HW-504).
- 3 mini servo (SG90 or SG90s).
- Stepper motor (28BYJ-48).
- Stepper motor driver (DVR8825).
- DC DC step-down converter.

---

Robotic arm project [mk3](https://www.thingiverse.com/thing:2838859) with [these](https://www.thingiverse.com/thing:3415531) improvements.

---

## Wiring Diagram

| Component           | ESP32-C3 GPIO | Note                        |
|---------------------|---------------|-----------------------------|
| Servo 1 (shoulder)  | GPIO5         | PWM (orange wire)           |
| Servo 2 (elbow)     | GPIO6         | PWM (orange wire)           |
| Servo 3 (gripper)   | GPIO7         | PWM (orange wire)           |
| Joystick 1 X        | GPIO0         | ADC                         |
| Joystick 1 Y        | GPIO1         | ADC                         |
| Joystick 2 X        | GPIO2         | ADC                         |
| Joystick 2 Y        | GPIO3         | ADC                         |
| Servo power         | 5V            | From DC-DC converter        |
|---------------------|---------------| ----------------------------- |

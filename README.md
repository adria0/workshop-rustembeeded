# Rust embeeded workshop

![](rust-embeeded.jpg)
 
This is the result of a nice rust-embeeded workshop organized by @dhole in Barcelona.
The documentation is in https://github.com/Dhole/rust-bluepill-doc

Connections for display

- STM32F103C8::GND -> SSD1306::GND
- STM32F103C8::3.3 -> SSD1306::VDD
- STM32F103C8::B8  -> SSD1306::SCK
- STM32F103C8::B9  -> SSD1306::SDA

Connections for sound

- STM32F103C8::A0 -> IRF520::GATE (first)
- IRF520::DRAIN (second) -> R40OHM
- IRF520::SOURCE (third) -> STM32F103C8::GND
- R40OHM -> SPEAKER
- SPEAKER -> STM32F103C8::3.3

To connect to computer:

- ST-LINK V2 <-> STM32F103C8
  - GND
  - SWCLK
  - SWD
  - 3V3



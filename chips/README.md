Tock Chips
==========

The `/chips` folder contains the list of microcontrollers supported by Tock.
Each MCU folder contains the hardware peripheral drivers for that MCU.



HIL Support
-----------

<!--START OF HIL SUPPORT-->

| HIL                                     | arty_e21 | cc26x2 | e310x | nrf52 | sam4l | stm32f4xx | ibex |
|-----------------------------------------|----------|--------|-------|-------|-------|-----------|------|
| adc::Adc                                |          |        |       | ✓     | ✓     |           |      |
| adc::AdcHighSpeed                       |          |        |       |       | ✓     |           |      |
| analog_comparator::AnalogComparator     |          |        |       |       | ✓     |           |      |
| ble_advertising::BleAdvertisementDriver |          |        |       | ✓     |       |           |      |
| ble_advertising::BleConfig              |          |        |       | ✓     |       |           |      |
| crc::CRC                                |          |        |       |       | ✓     |           |      |
| dac::DacChannel                         |          |        |       |       | ✓     |           |      |
| eic::ExternalInterruptController        |          |        |       |       | ✓     |           |      |
| entropy::Entropy32                      |          | ✓      |       | ✓     | ✓     |           |      |
| gpio::Interrupt                         | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✓    |
| gpio::InterruptPin                      | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✓    |
| gpio::Output                            | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✓    |
| gpio::Pin                               | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✓    |
| i2c::I2CMaster                          |          | ✓      |       | ✓     | ✓     |           | ✗ unimplemented    |
| i2c::I2CMasterSlave                     |          |        |       |       | ✓     |           |      |
| i2c::I2CSlave                           |          |        |       |       | ✓     |           |      |
| mod::Controller                         |          |        |       | ✓     | ✓     |           |      |
| pwm::Pwm                                |          |        |       | ✓     |       |           |      |
| radio::Radio                            |          |        |       | ✓     |       |           |      |
| radio::RadioConfig                      |          |        |       | ✓     |       |           |      |
| radio::RadioData                        |          |        |       | ✓     |       |           |      |
| sensors::TemperatureDriver              |          |        |       | ✓     |       |           |      |
| spi::SpiMaster                          |          |        |       | ✓     | ✓     | ✓         | ✗ unimplemented    |
| spi::SpiSlave                           |          |        |       |       | ✓     |           | ✗ unimplemented    |
| symmetric_encryption::AES128            |          |        |       | ✓     | ✓     |           | ✗ unimplemented    |
| symmetric_encryption::AES128CBC         |          |        |       | ✓     | ✓     |           |      |
| symmetric_encryption::AES128CCM         |          |        |       | ✓     |       |           |      |
| symmetric_encryption::AES128Ctr         |          |        |       | ✓     | ✓     |           | ✗ unimplemented    |
| time::Alarm                             |          | ✓      |       | ✓     | ✓     | ✓         | ✓    |
| time::Frequency                         |          | ✓      |       | ✓     |       |           | ✓    |
| time::Time                              |          | ✓      |       | ✓     | ✓     | ✓         | ✓    |
| uart::Configure                         | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✓    |
| uart::Receive                           | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✗ unimplemented    |
| uart::ReceiveAdvanced                   |          |        |       |       | ✓     |           |      |
| uart::Transmit                          | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✓    |
| uart::Uart                              | ✓        | ✓      | ✓     | ✓     | ✓     | ✓         | ✓    |
| uart::UartAdvanced                      |          |        |       |       | ✓     |           |      |
| uart::UartData                          | ✓        | ✓      | ✓     | ✓     |       | ✓         | ✓    |
| usb::UsbController                      |          |        |       |       | ✓     |           |      |
| watchdog::Watchdog                      |          |        |       |       | ✓     |           |      |

<!--END OF HIL SUPPORT-->



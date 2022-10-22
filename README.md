# i2c-software-gpio

Bitbanging I2C GPIO using sysfs with embedded-hal traits implementation.

## Usage
### I2C Bus Scan
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let speed = 1000000;
    let scl = Pin::new(16);
    let sda = Pin::new(15);
    let i2c_gpio = I2cGPIO::new(scl, sda, speed);

   for addr in 0..128 {
        i2c_gpio.start()?;
        if i2c_gpio.write_byte(addr << 1 | 1)? == 0 {
            println!("Device found at: {:#04x}", addr);
        }
        i2c_gpio.stop()?;
    }
}
```

## References
- [I²C - Wikipedia - Example of bit-banging the I2C protocol](https://en.wikipedia.org/wiki/I%C2%B2C#Example_of_bit-banging_the_I.C2.B2C_master_protocol)

- [bitbang-hal](https://github.com/sajattack/bitbang-hal)

# License
This project is released under the [Unlicense](https://unlicense.org/).

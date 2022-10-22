use std::{thread::sleep, time::Duration};

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use sysfs_gpio::{Direction, Pin};

pub struct I2cGPIO {
    scl: Pin,
    sda: Pin,
    speed: u64,
}

impl I2cGPIO {
    pub fn new(scl: Pin, sda: Pin, speed: u64) -> Self {
        scl.export().unwrap();
        sda.export().unwrap();
        I2cGPIO {
            scl: scl,
            sda: sda,
            speed: speed,
        }
    }

    pub fn start(&self) -> Result<(), sysfs_gpio::Error> {
        if self.sda_release()? == 0 {
            self.reset()?;
        }
        self.wait_scl_release()?;

        self.sda_pull()?;
        self.scl_pull()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), sysfs_gpio::Error> {
        self.wait_scl_release()?;
        if self.sda_release()? == 0 {
            self.reset()?;
        }

        Ok(())
    }

    pub fn reset(&self) -> Result<(), sysfs_gpio::Error> {
        let mut i = 0;
        self.sda_release()?;
        self.sda.set_direction(Direction::In)?;
        loop {
            for _i in 0..10 {
                self.scl_pull()?;
                self.scl_release()?;
            }
            i += 1;
            if i > 100 {
                break;
            }
            sleep(Duration::from_millis(10));
            if self.sda.get_value()? == 1 { break ;} 
        }

        self.scl_pull()?;
        self.sda_pull()?;

        self.stop()?;

        Ok(())

    }

    pub fn read_bit(&self) -> Result<u8, sysfs_gpio::Error> {
        self.sda_release()?;
        self.wait_scl_release()?;
        let value = self.sda.get_value()?;
        self.scl_pull()?;
        self.sda_pull()?;

        Ok(value)

    }

    pub fn read_byte(&self, send_ack: bool) -> Result<u8, sysfs_gpio::Error> {
        let mut byte: u8 = 0x00;

        for _i in 0..8 {
            byte = (byte << 1) | self.read_bit()?;
        }

        if send_ack {
            self.write_bit(0)?;
        } else {
            self.write_bit(1)?;
        }

        Ok(byte)
    }

    pub fn write_bit(&self, bit: u8) -> Result<(), sysfs_gpio::Error> {
        if bit == 1 {
            self.sda_release()?;
        } else {
            self.sda_pull()?;
        }

        self.wait_scl_release()?;
        self.scl_pull()?;

        self.sda_pull()?;

        Ok(())
    }

    pub fn write_byte(&self, byte: u8) -> Result<u8, sysfs_gpio::Error> {
        for bit_offset in 0..8 {
            let out_bit = (byte >> (7 - bit_offset)) & 0b1;

            if out_bit == 1 {
                self.sda_release()?;
            } else {
                self.sda_pull()?;
            }

            self.wait_scl_release()?;

            self.scl_pull()?;
            self.sda_pull()?;
        }

        Ok(self.read_bit()?)
    }


    fn read_slice(&mut self, input: &mut [u8]) -> Result<(), sysfs_gpio::Error> {
        for i in 0..input.len() {
            let send_ack = i != (input.len() - 1);
            input[i] = self.read_byte(send_ack)?;
        }
        Ok(())
    }

    fn write_slice(&mut self, output: &[u8]) -> Result<(), sysfs_gpio::Error> {
        for byte in output {
            let ack = self.write_byte(*byte)?;
            dbg!("ACK = {}", ack);
        }
        Ok(())
    }

    fn scl_pull(&self) -> Result<(), sysfs_gpio::Error> {
        self.scl.set_direction(Direction::Out)?;
        self.scl.set_value(0)?;
        sleep(Duration::from_micros(1000000/self.speed/2));
        Ok(())
    }

    fn sda_pull(&self) -> Result<(), sysfs_gpio::Error> {
        self.sda.set_direction(Direction::Out)?;
        self.sda.set_value(0)?;
        sleep(Duration::from_micros(1000000/self.speed/2));
        Ok(())
    }

    fn scl_release(&self) -> Result<u8, sysfs_gpio::Error> {
        self.scl.set_direction(Direction::In)?;
        sleep(Duration::from_micros(1000000/self.speed/2));
        let value = self.scl.get_value()?;
        Ok(value)
    }

    fn wait_scl_release(&self) -> Result<(), sysfs_gpio::Error> {
        self.scl.set_direction(Direction::In)?;
        sleep(Duration::from_micros(1000000/self.speed/2));
        while self.scl.get_value()? == 0 {
            sleep(Duration::from_millis(2000));
            break;
        }
        sleep(Duration::from_micros(1000000/self.speed/2));
        Ok(())
    }

    fn sda_release(&self) -> Result<u8, sysfs_gpio::Error> {
        self.sda.set_direction(Direction::In)?;
        sleep(Duration::from_micros(1000000/self.speed/2));
        let value = self.sda.get_value()?;

        Ok(value)

    }

}


impl Write for I2cGPIO {
    type Error = sysfs_gpio::Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.start()?;

        let ack = self.write_byte((address << 1) | 0x0)?;
        dbg!("ACK = {}", ack);

        self.write_slice(bytes)?;

        self.stop()?;

        Ok(())
        
    }
}

impl Read for I2cGPIO {
    type Error = sysfs_gpio::Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.start()?;

        let ack = self.write_byte((address << 1) | 0x1)?;
        dbg!("ACK = {}", ack);

        self.read_slice(buffer)?;

        self.stop()?;

        Ok(())

    }
}

impl WriteRead for I2cGPIO {
    type Error = sysfs_gpio::Error;

    fn write_read(
            &mut self,
            address: u8,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error> {

        self.start()?;

        let mut ack = self.write_byte((address << 1) | 0x0)?;
        dbg!("ACK = {}", ack);

        self.write_slice(bytes)?;

        self.start()?;

        ack = self.write_byte((address << 1) | 0x1)?;
        dbg!("ACK = {}", ack);

        self.read_slice(buffer)?;

        self.stop()?;

        Ok(())
    }
}

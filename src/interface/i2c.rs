use super::{SensorCommon, SensorInterface, PACKET_HEADER_LENGTH};
use crate::Error;
use embedded_hal::blocking::delay::DelayMs;

#[cfg(feature = "defmt-03")]
use defmt::println;

/// the i2c address normally used by BNO080
pub const DEFAULT_ADDRESS: u8 = 0x4A;
/// alternate i2c address for BNO080
pub const ALTERNATE_ADDRESS: u8 = 0x4B;

/// Length of our receive buffer:
/// Note that this likely needs to be < 256 to accommodate underlying HAL
const SEG_RECV_BUF_LEN: usize = 240;
const MAX_SEGMENT_READ: usize = SEG_RECV_BUF_LEN;

pub struct I2cInterface<I2C> {
    /// i2c port
    i2c_port: I2C,
    /// address for i2c communications with the sensor hub
    address: u8,
    /// buffer for receiving segments of packets from the sensor hub
    seg_recv_buf: [u8; SEG_RECV_BUF_LEN],

    /// number of packets received
    received_packet_count: usize,
}

impl<I2C, CommE> I2cInterface<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = CommE>
        + embedded_hal::blocking::i2c::Read<Error = CommE>
        + embedded_hal::blocking::i2c::WriteRead<Error = CommE>,
{
    pub fn default(i2c: I2C) -> Self {
        Self::new(i2c, DEFAULT_ADDRESS)
    }

    pub fn alternate(i2c: I2C) -> Self {
        Self::new(i2c, ALTERNATE_ADDRESS)
    }

    pub fn new(i2c: I2C, addr: u8) -> Self {
        Self {
            i2c_port: i2c,
            address: addr,
            seg_recv_buf: [0; SEG_RECV_BUF_LEN],
            received_packet_count: 0,
        }
    }

    pub fn free(self) -> I2C {
        self.i2c_port
    }

    fn read_packet_header(&mut self) -> Result<(), Error<CommE, ()>> {
        self.zero_recv_packet_header();
        self.i2c_port
            .read(self.address, &mut self.seg_recv_buf[..PACKET_HEADER_LENGTH])
            .map_err(Error::Comm)?;

        Ok(())
    }

    /// Read the remainder of the packet after the packet header, if any
    fn read_sized_packet(
        &mut self,
        total_packet_len: usize,
        packet_recv_buf: &mut [u8],
    ) -> Result<usize, Error<CommE, ()>> {
        let mut remaining_body_len: usize =
            total_packet_len - PACKET_HEADER_LENGTH;
        let mut already_read_len: usize = 0;

        // zero packet header receive buffer
        for byte in &mut packet_recv_buf[..PACKET_HEADER_LENGTH] {
            *byte = 0;
        }

        // #[cfg(feature = "defmt-03")]
        // println!("r.t {}", total_packet_len);

        if total_packet_len < MAX_SEGMENT_READ {
            //read directly into the provided receive buffer
            if total_packet_len > 0 {
                self.i2c_port
                    .read(
                        self.address,
                        &mut packet_recv_buf[..total_packet_len],
                    )
                    .map_err(Error::Comm)?;
                already_read_len = total_packet_len;
            }
        } else {
            while remaining_body_len > 0 {
                let whole_segment_length =
                    remaining_body_len + PACKET_HEADER_LENGTH;
                let segment_read_len =
                    if whole_segment_length > MAX_SEGMENT_READ {
                        MAX_SEGMENT_READ
                    } else {
                        whole_segment_length
                    };
                // #[cfg(feature = "defmt-03")]
                // println!("r.s {:x} {}", self.address, segment_read_len);

                self.zero_recv_packet_header();
                self.i2c_port
                    .read(
                        self.address,
                        &mut self.seg_recv_buf[..segment_read_len],
                    )
                    .map_err(Error::Comm)?;

                let promised_packet_len = SensorCommon::parse_packet_header(
                    &self.seg_recv_buf[..PACKET_HEADER_LENGTH],
                );
                if promised_packet_len <= PACKET_HEADER_LENGTH {
                    #[cfg(feature = "defmt-03")]
                    println!("WTFFF {}", promised_packet_len);
                    return Ok(0);
                }

                //if we've never read any segments, transcribe the first packet header;
                //otherwise, just transcribe the segment body (no header)
                let transcribe_start_idx = if already_read_len > 0 {
                    PACKET_HEADER_LENGTH
                } else {
                    0
                };
                let transcribe_len = if already_read_len > 0 {
                    segment_read_len - PACKET_HEADER_LENGTH
                } else {
                    segment_read_len
                };
                packet_recv_buf
                    [already_read_len..already_read_len + transcribe_len]
                    .copy_from_slice(
                        &self.seg_recv_buf[transcribe_start_idx
                            ..transcribe_start_idx + transcribe_len],
                    );
                already_read_len += transcribe_len;

                let body_read_len = segment_read_len - PACKET_HEADER_LENGTH;
                remaining_body_len -= body_read_len;
            }
        }

        Ok(already_read_len)
    }

    fn zero_recv_packet_header(&mut self) {
        Self::zero_buffer(&mut self.seg_recv_buf[..PACKET_HEADER_LENGTH]);
    }

    fn zero_buffer(buf: &mut [u8]) {
        for byte in buf.as_mut() {
            *byte = 0;
        }
    }
}

impl<I2C, CommE> SensorInterface for I2cInterface<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = CommE>
        + embedded_hal::blocking::i2c::Read<Error = CommE>
        + embedded_hal::blocking::i2c::WriteRead<Error = CommE>,
{
    type SensorError = Error<CommE, ()>;

    fn requires_soft_reset(&self) -> bool {
        true
    }

    fn setup(
        &mut self,
        delay_source: &mut impl DelayMs<u8>,
    ) -> Result<(), Self::SensorError> {
        // #[cfg(feature = "defmt-03")]
        // println!("i2c setup");
        delay_source.delay_ms(5);
        Ok(())
    }

    fn write_packet(&mut self, packet: &[u8]) -> Result<(), Self::SensorError> {
        #[cfg(feature = "defmt-03")]
        println!("w {:x} {}", self.address, packet.len());
        self.i2c_port
            .write(self.address, &packet)
            .map_err(Error::Comm)?;
        Ok(())
    }

    fn read_with_timeout(
        &mut self,
        recv_buf: &mut [u8],
        delay_source: &mut impl DelayMs<u8>,
        max_ms: u8,
    ) -> Result<usize, Self::SensorError> {
        let mut total_delay: u8 = 0;
        while total_delay < max_ms {
            match self.read_packet(recv_buf) {
                Ok(read_size) => {
                    if 0 == read_size {
                        // no data available yet...wait a while longer
                        delay_source.delay_ms(1);
                        total_delay += 1;
                    } else {
                        return Ok(read_size);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(0)
    }

    /// Read one packet into the receive buffer
    fn read_packet(
        &mut self,
        recv_buf: &mut [u8],
    ) -> Result<usize, Self::SensorError> {
        // #[cfg(feature = "defmt-03")]
        // println!("rpkt");

        self.read_packet_header()?;
        let packet_len = SensorCommon::parse_packet_header(
            &self.seg_recv_buf[..PACKET_HEADER_LENGTH],
        );

        // if packet_len == 0 {
        //     #[cfg(feature = "defmt-03")]
        //     println!("eh {:x?}", &self.seg_recv_buf[..PACKET_HEADER_LENGTH]);
        // }

        let received_len = if packet_len > PACKET_HEADER_LENGTH {
            self.read_sized_packet(packet_len, recv_buf)?
        } else {
            packet_len
        };

        if packet_len > 0 {
            self.received_packet_count += 1;
            //let _ = SensorCommon::parse_packet_header(&recv_buf[..packet_len]);
        }

        Ok(received_len)
    }

    fn send_and_receive_packet(
        &mut self,
        send_buf: &[u8],
        recv_buf: &mut [u8],
    ) -> Result<usize, Self::SensorError> {
        // Cannot use write_read with bno080,
        // because it does not support repeated start with i2c.

        self.i2c_port
            .write(self.address, send_buf)
            .map_err(Error::Comm)?;

        self.zero_recv_packet_header();
        //stall before attempted read?
        Self::zero_buffer(recv_buf);

        self.i2c_port
            .read(self.address, &mut self.seg_recv_buf[..PACKET_HEADER_LENGTH])
            .map_err(Error::Comm)?;

        let packet_len = SensorCommon::parse_packet_header(
            &self.seg_recv_buf[..PACKET_HEADER_LENGTH],
        );

        let received_len = if packet_len > PACKET_HEADER_LENGTH {
            self.read_sized_packet(packet_len, recv_buf)?
        } else {
            packet_len
        };
        if packet_len > 0 {
            self.received_packet_count += 1;
        }

        Ok(received_len)
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::i2c::DEFAULT_ADDRESS;
    use crate::interface::mock_i2c_port::FakeI2cPort;
    use crate::interface::I2cInterface;
    use crate::wrapper::BNO080;

    // #[test]
    // fn test_multi_segment_receive_packet() {
    //     let mut mock_i2c_port = FakeI2cPort::new();

    //     let packet = ADVERTISING_PACKET_FULL;
    //     mock_i2c_port.add_available_packet(&packet);

    //     let mut shub = BNO080::new_with_interface(I2cInterface::new(
    //         mock_i2c_port,
    //         DEFAULT_ADDRESS,
    //     ));
    //     let rc = shub.receive_packet();

    //     assert!(rc.is_ok());
    //     let next_packet_size = rc.unwrap_or(0);
    //     assert_eq!(next_packet_size, packet.len(), "wrong length");
    // }

    //TODO test failing due to bug in mock_i2c_port
    // #[test]
    // fn test_receive_under() {
    //     let mut mock_i2c_port = FakeI2cPort::new();
    //
    //     let packet: [u8; 3] = [0; 3];
    //     mock_i2c_port.add_available_packet(&packet);
    //
    //     let mut shub = BNO080::new_with_interface(
    //         I2cInterface::new(mock_i2c_port, DEFAULT_ADDRESS));
    //     let rc = shub.receive_packet();
    //
    //     assert!(rc.is_err());
    // }

    // Actual advertising packet received from sensor:
    pub const ADVERTISING_PACKET_FULL: [u8; 276] = [
        0x14, 0x81, 0x00, 0x01, 0x00, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x80,
        0x06, 0x31, 0x2e, 0x30, 0x2e, 0x30, 0x00, 0x02, 0x02, 0x00, 0x01, 0x03,
        0x02, 0xff, 0x7f, 0x04, 0x02, 0x00, 0x01, 0x05, 0x02, 0xff, 0x7f, 0x08,
        0x05, 0x53, 0x48, 0x54, 0x50, 0x00, 0x06, 0x01, 0x00, 0x09, 0x08, 0x63,
        0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c, 0x00, 0x01, 0x04, 0x01, 0x00, 0x00,
        0x00, 0x08, 0x0b, 0x65, 0x78, 0x65, 0x63, 0x75, 0x74, 0x61, 0x62, 0x6c,
        0x65, 0x00, 0x06, 0x01, 0x01, 0x09, 0x07, 0x64, 0x65, 0x76, 0x69, 0x63,
        0x65, 0x00, 0x01, 0x04, 0x02, 0x00, 0x00, 0x00, 0x08, 0x0a, 0x73, 0x65,
        0x6e, 0x73, 0x6f, 0x72, 0x68, 0x75, 0x62, 0x00, 0x06, 0x01, 0x02, 0x09,
        0x08, 0x63, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c, 0x00, 0x06, 0x01, 0x03,
        0x09, 0x0c, 0x69, 0x6e, 0x70, 0x75, 0x74, 0x4e, 0x6f, 0x72, 0x6d, 0x61,
        0x6c, 0x00, 0x07, 0x01, 0x04, 0x09, 0x0a, 0x69, 0x6e, 0x70, 0x75, 0x74,
        0x57, 0x61, 0x6b, 0x65, 0x00, 0x06, 0x01, 0x05, 0x09, 0x0c, 0x69, 0x6e,
        0x70, 0x75, 0x74, 0x47, 0x79, 0x72, 0x6f, 0x52, 0x76, 0x00, 0x80, 0x06,
        0x31, 0x2e, 0x31, 0x2e, 0x30, 0x00, 0x81, 0x64, 0xf8, 0x10, 0xf5, 0x04,
        0xf3, 0x10, 0xf1, 0x10, 0xfb, 0x05, 0xfa, 0x05, 0xfc, 0x11, 0xef, 0x02,
        0x01, 0x0a, 0x02, 0x0a, 0x03, 0x0a, 0x04, 0x0a, 0x05, 0x0e, 0x06, 0x0a,
        0x07, 0x10, 0x08, 0x0c, 0x09, 0x0e, 0x0a, 0x08, 0x0b, 0x08, 0x0c, 0x06,
        0x0d, 0x06, 0x0e, 0x06, 0x0f, 0x10, 0x10, 0x05, 0x11, 0x0c, 0x12, 0x06,
        0x13, 0x06, 0x14, 0x10, 0x15, 0x10, 0x16, 0x10, 0x17, 0x00, 0x18, 0x08,
        0x19, 0x06, 0x1a, 0x00, 0x1b, 0x00, 0x1c, 0x06, 0x1d, 0x00, 0x1e, 0x10,
        0x1f, 0x00, 0x20, 0x00, 0x21, 0x00, 0x22, 0x00, 0x23, 0x00, 0x24, 0x00,
        0x25, 0x00, 0x26, 0x00, 0x27, 0x00, 0x28, 0x0e, 0x29, 0x0c, 0x2a, 0x0e,
    ];
}

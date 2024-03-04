/*
Copyright (c) 2020 Todd Stellanova
LICENSE: BSD3 (see LICENSE file)
*/

#![no_std]

pub mod interface;
pub mod wrapper;
pub mod sensorhub;

#[cfg(not(feature = "defmt-03"))]
mod dummy_defmt;

#[cfg(feature = "defmt-03")]
use defmt_03 as defmt;

#[cfg(not(feature = "defmt-03"))]
use dummy_defmt as defmt;

/// Errors in this crate
#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Error<CommE, PinE> {
    /// Sensor communication error
    Comm(CommE),
    /// Pin setting error
    Pin(PinE),
    /// The sensor is not responding
    SensorUnresponsive,
}

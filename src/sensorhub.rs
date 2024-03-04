use paste::paste;

#[cfg(feature = "defmt-03")]
use defmt_03 as defmt;

macro_rules! qpoint_impl {
    ( $(($qpoint:literal <> $name:ident));* $(;)? ) => {
        #[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
        #[derive(Debug)]
        pub enum QPoint {
            None,
            $($name),*
        }
        impl QPoint {
            $(
                paste! {
                    pub fn [<q $qpoint _to_f32>](q_val: i16) -> f32 {
                        (q_val as f32) * ((1 << $qpoint) as f32)
                    }
                }
            )*

            pub fn to_f32(&self, q_val: i16) -> f32 {
                match self {
                    QPoint::None => 0.0,
                    $(
                        QPoint::$name => paste! { QPoint::[<q $qpoint _to_f32>](q_val) },
                    )*
                }
            }
        }
            
    };
}

/// Common Dynamic Feature Report as found in the SH2 Reference Manual (6.5.2)
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct FeatureReportConfig {
    feature: Feature,
    flags: u8,
    change_sensitivity: u16,
    report_interval: u32,
    batch_interval: u32,
    sensor_specific_config: u32,
}

#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Feature {
    /// Reports values as ADCs
    RawAccelerometer = 0x14,

    /// Reports values as m/s^2
    Accelerometer = 0x01,

    /// Reports values as m/s^2
    LinearAcceleration = 0x04,

    /// Reports values as m/s^2
    Gravity = 0x06,

    /// Reports values as ADCs
    RawGyroscope = 0x15,

    /// Reports values as rad/s
    GyroscopeCalibrated = 0x02,

    /// Reports values as rad/s, also reports drift estimation as rad/s, with Q point of 9
    GyroscopeUncalibrated = 0x07,

    /// Reports values as ADCs
    RawMagnetometer = 0x16,

    /// Reports values as uTesla
    MagneticFieldCalibrated = 0x03,

    /// Reports values as uTesla, also reports drift estimation as uTesla, with Q point of 4
    MagneticFieldUncalibrated = 0x0F,

    /// Reports values as unit quaternion, also reports estimation of heading accuracy, with Q point of 12
    RotationVector = 0x05,

    /// Reports values as unit quaternion
    GameRotationVector = 0x08,

    /// Reports values as unit quaternion, also reports estimation of heading accuracy, with Q point as 12
    GeomagneticRotationVector = 0x09,

    /// Reports values as hectopascals
    Pressure = 0x0A,

    AmbientLight = 0x0B,
    Humidity = 0x0C,
    Proximity = 0x0D,
    Temperature = 0x0E,
    TapDetector = 0x10,
    StepDetector = 0x18,
    StepCounter = 0x11,
    SignificantMotion = 0x12,
    StabilityClassifier = 0x13,
    ShakeDetector = 0x19,
    FlipDetector = 0x1A,
    PickupDetector = 0x1B,
    StabilityDetector = 0x1C,
    PersonalActivityClassifier = 0x1E,
    SleepDetector = 0x1F,
    TiltDetector = 0x20,
    PocketDetector = 0x21,
    CircleDetector = 0x22,
    HeartRateMonitor = 0x23,
    ArVrStabilisedRotationVector = 0x28,
    ArVrStabilisedGameRotationVector = 0x29,
    GyroIntegratedRotationVector = 0x2A,
}

impl From<Feature> for QPoint {
    fn from(f: Feature) -> Self {
        match f {
            Feature::RawAccelerometer => QPoint::None,
            Feature::Accelerometer => QPoint::Eight,
            Feature::LinearAcceleration => QPoint::Eight,
            Feature::Gravity => QPoint::Eight,
            Feature::RawGyroscope => QPoint::None,
            Feature::GyroscopeCalibrated => QPoint::Nine,
            Feature::GyroscopeUncalibrated => QPoint::Nine,
            Feature::RawMagnetometer => QPoint::None,
            Feature::MagneticFieldCalibrated => QPoint::Four,
            Feature::MagneticFieldUncalibrated => QPoint::Four,
            Feature::RotationVector => QPoint::Fourteen,
            Feature::GameRotationVector => QPoint::Fourteen,
            Feature::GeomagneticRotationVector => QPoint::Fourteen,
            Feature::Pressure => todo!(),
            Feature::AmbientLight => todo!(),
            Feature::Humidity => todo!(),
            Feature::Proximity => todo!(),
            Feature::Temperature => todo!(),
            Feature::TapDetector => todo!(),
            Feature::StepDetector => todo!(),
            Feature::StepCounter => todo!(),
            Feature::SignificantMotion => todo!(),
            Feature::StabilityClassifier => todo!(),
            Feature::ShakeDetector => todo!(),
            Feature::FlipDetector => todo!(),
            Feature::PickupDetector => todo!(),
            Feature::StabilityDetector => todo!(),
            Feature::PersonalActivityClassifier => todo!(),
            Feature::SleepDetector => todo!(),
            Feature::TiltDetector => todo!(),
            Feature::PocketDetector => todo!(),
            Feature::CircleDetector => todo!(),
            Feature::HeartRateMonitor => todo!(),
            Feature::ArVrStabilisedRotationVector => todo!(),
            Feature::ArVrStabilisedGameRotationVector => todo!(),
            Feature::GyroIntegratedRotationVector => todo!(),
        }
    }
}

impl core::default::Default for Feature {
    fn default() -> Self {
        Feature::Accelerometer
    }
}

qpoint_impl! {
    (4  <> Four);
    (8  <> Eight);
    (9  <> Nine);
    (12 <> Twelve);
    (14 <> Fourteen);
    (20 <> Twenty);
}

#[repr(u8)]
pub enum FeatureFlags {
    ChangeSensitivity = 0x01,
    WakeUp = 0x02,
    AlwaysOn = 0x04,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn q_point() {
        let qp: QPoint = Feature::Accelerometer.into();
        qp.to_f32(1);
    }
}

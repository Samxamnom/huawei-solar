use std::{thread::sleep, time::Duration};

use modbus::Transport;

pub mod registers {
    use std::fmt::Display;
    use num::pow::Pow;

    use bit_vec::BitVec;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum RegisterError {
        #[error("Failed to convert: {0}")]
        ValueConversion(String),
    }
    pub trait RegisterType {
        fn convert(vec: &[u16]) -> Option<Self>
        where
            Self: Sized;
    }
    impl RegisterType for String {
        fn convert(vec: &[u16]) -> Option<Self> {
            String::from_utf8(vec.into_iter().flat_map(|u| u.to_be_bytes()).collect()).ok()
        }
    }
    impl RegisterType for u16 {
        fn convert(vec: &[u16]) -> Option<Self> {
            vec.first().copied()
        }
    }
    #[test]
    fn convert_u16() {
        assert_eq!(u16::convert(&vec![u16::MIN]), Some(u16::MIN));
        assert_eq!(u16::convert(&vec![0u16]), Some(0u16));
        assert_eq!(u16::convert(&vec![1u16]), Some(1u16));
        assert_eq!(u16::convert(&vec![u16::MAX]), Some(u16::MAX));
    }
    impl RegisterType for u32 {
        fn convert(vec: &[u16]) -> Option<Self> {
            Some((vec[0] as u32) << 16 | vec[1] as u32)
        }
    }
    #[test]
    fn convert_u32() {
        assert_eq!(u32::convert(&vec![u16::MIN, u16::MIN]), Some(u32::MIN));
        assert_eq!(u32::convert(&vec![0u16, 0u16]), Some(0u32));
        assert_eq!(
            u32::convert(&vec![0x000Fu16, 0xF0FFu16]),
            Some(0x000FF0FFu32)
        );
        assert_eq!(u32::convert(&vec![u16::MAX, u16::MAX]), Some(u32::MAX));
    }

    impl RegisterType for i16 {
        fn convert(vec: &[u16]) -> Option<Self> {
            vec.first().map(|v| *v as i16)
        }
    }

    #[test]
    fn convert_i16() {
        assert_eq!(i16::convert(&vec![0u16]), Some(0i16));
        assert_eq!(i16::convert(&vec![1u16]), Some(1i16));
        assert_eq!(i16::convert(&vec![u16::MAX]), Some(-1i16));
    }

    impl RegisterType for i32 {
        fn convert(vec: &[u16]) -> Option<Self> {
            Some((vec[0] as i32) << 16 | vec[1] as i32)
        }
    }
    #[test]
    fn convert_i32() {
        assert_eq!(
            <i32 as RegisterType>::convert(&vec![0u16, 0u16]),
            Some(0i32)
        );
        assert_eq!(
            <i32 as RegisterType>::convert(&vec![0x000Fu16, 0xF0FFu16]),
            Some(0x000FF0FFi32)
        );
        assert_eq!(
            <i32 as RegisterType>::convert(&vec![u16::MAX, u16::MAX]),
            Some(-1i32)
        );
    }

    // TODO
    // BitVec read bits from MSB to LSB -- probably wrong direction
    impl RegisterType for BitVec {
        fn convert(vec: &[u16]) -> Option<Self> {
            let bytes = vec
                .iter()
                .flat_map(|&num| num.to_be_bytes())
                .collect::<Vec<u8>>();

            Some(BitVec::from_bytes(&bytes))
        }
    }

    #[test]
    #[rustfmt::skip]
    fn convert_bf() {
        assert!(BitVec::convert(&vec![0u16, 0u16]).unwrap().capacity() >= 32);
        assert_eq!(BitVec::convert(&vec![0u16, 0u16]).unwrap()[0], false);
        assert_eq!(BitVec::convert(&vec![8u16, 0u16]).unwrap()[12], true);
    }

    #[derive(Debug)]
    pub enum Type {
        U16,
        U32,
        I16,
        I32,
        BF,
        STR,
    }
    impl Type {
        #[rustfmt::skip]
        pub fn convert(&self, val: &Vec<u16>) -> Option<Value> {
            match self {
                Type::U16 => Some(Value::U16(   u16::convert(val)?)),
                Type::U32 => Some(Value::U32(   u32::convert(val)?)),
                Type::I16 => Some(Value::I16(   i16::convert(val)?)),
                Type::I32 => Some(Value::I32(   i32::convert(val)?)),
                Type::BF  => Some(Value::BF (BitVec::convert(val)?)),
                Type::STR => Some(Value::STR(String::convert(val)?)),
            }
        }
    }

    #[derive(Debug)]
    pub enum Value {
        U16(u16),
        U32(u32),
        I16(i16),
        I32(i32),
        BF(BitVec),
        STR(String),
    }

    pub struct RegValue<'a, 'b: 'a> {
        pub reg: &'a Register<'b>,
        pub val: Value,
    }
    impl<'a, 'b> RegValue<'a, 'b> {
        pub fn to_float(&self) -> Result<f64, RegisterError> {
            match self.val {
                Value::U16(v) => Ok(v as f64 * 10f64.pow(self.reg.gain)),
                Value::U32(v) => Ok(v as f64 * 10f64.pow(self.reg.gain)),
                Value::I16(v) => Ok(v as f64 * 10f64.pow(self.reg.gain)),
                Value::I32(v) => Ok(v as f64 * 10f64.pow(self.reg.gain)),
                Value::BF(_) => Err(RegisterError::ValueConversion(
                    "Cannot convert bit field to float".to_string(),
                )),
                Value::STR(_) => Err(RegisterError::ValueConversion(
                    "Cannot convert string to float".to_string(),
                )),
            }
        }
        pub fn to_string(self) -> Result<String, RegisterError> {
            match self.val {
                Value::STR(v) => Ok(v),
                default => Err(RegisterError::ValueConversion(format!(
                    "Cannot convert {:?} to String",
                    default
                ))),
            }
        }
        pub fn to_i32(&self) -> Result<i32, RegisterError> {
            match &self.val {
                Value::I32(v) => Ok(*v),
                default => Err(RegisterError::ValueConversion(format!(
                    "Cannot convert {:?} to i32",
                    default
                ))),
            }
        }
        pub fn to_u32(&self) -> Result<u32, RegisterError> {
            match &self.val {
                Value::U32(v) => Ok(*v),
                default => Err(RegisterError::ValueConversion(format!(
                    "Cannot convert {:?} to u32",
                    default
                ))),
            }
        }
        pub fn to_u16(&self) -> Result<u16, RegisterError> {
            match &self.val {
                Value::U16(v) => Ok(*v),
                default => Err(RegisterError::ValueConversion(format!(
                    "Cannot convert {:?} to u16",
                    default
                ))),
            }
        }
        pub fn to_i16(&self) -> Result<i16, RegisterError> {
            match &self.val {
                Value::I16(v) => Ok(*v),
                default => Err(RegisterError::ValueConversion(format!(
                    "Cannot convert {:?} to i16",
                    default
                ))),
            }
        }
    }

    impl<'a, 'b> Display for RegValue<'a, 'b> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.val {
                Value::U16(_) | Value::U32(_) | Value::I16(_) | Value::I32(_) => write!(
                    f,
                    "{}{}",
                    self.to_float().unwrap(),
                    self.reg.unit.unwrap_or("")
                ),
                Value::BF(v) => write!(f, "{v:?}"),
                Value::STR(v) => write!(f, "{v}"),
            }
        }
    }

    #[derive(Debug)]
    pub enum Access {
        RO,
        WO,
        RW,
    }

    #[derive(Debug)]
    pub struct Register<'a /* , T: RegisterType */> {
        pub address: u16,
        pub quantity: u8,
        pub gain: u8,
        pub unit: Option<&'a str>,
        pub access: Access,
        pub typ: Type,
        pub name: &'a str,
        // typ: std::marker::PhantomData<T>,
    }

    pub use nofmt::*;
    #[rustfmt::skip]
    mod nofmt {
        // use bit_vec::BitVec;
        use super::{Access, Type, Register};

        pub const MODEL:                            Register = Register { address: 30000, quantity: 15, gain: 0, unit: None        , access: Access::RO, typ: Type::STR, name: "MODEL"                            };
        pub const SN:                               Register = Register { address: 30015, quantity: 10, gain: 0, unit: None        , access: Access::RO, typ: Type::STR, name: "SN"                               };
        pub const PN:                               Register = Register { address: 30025, quantity: 10, gain: 0, unit: None        , access: Access::RO, typ: Type::STR, name: "PN"                               };
        pub const MODEL_ID:                         Register = Register { address: 30070, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::U16, name: "MODEL_ID"                         };
        pub const NUMBER_OF_PV_STRINGS:             Register = Register { address: 30071, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::U16, name: "NUMBER_OF_PV_STRINGS"             };
        pub const NUMBER_OF_MPP_TRACKERS:           Register = Register { address: 30072, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::U16, name: "NUMBER_OF_MPP_TRACKERS"           };
        pub const RATED_POWER:                      Register = Register { address: 30073, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: Type::U32, name: "RATED_POWER"                      };
        pub const MAXIMUM_ACTIVE_POWER:             Register = Register { address: 30075, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: Type::U32, name: "MAXIMUM_ACTIVE_POWER"             };
        pub const MAXIMUM_APPARENT_POWER:           Register = Register { address: 30077, quantity:  2, gain: 3, unit: Some("kVA") , access: Access::RO, typ: Type::U32, name: "MAXIMUM_APPARENT_POWER"           };
        pub const MAXIMUM_REACTIVE_POWER_TO_GRID:   Register = Register { address: 30079, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO, typ: Type::I32, name: "MAXIMUM_REACTIVE_POWER_TO_GRID"   };
        pub const MAXIMUM_APPARENT_POWER_FROM_GRID: Register = Register { address: 30081, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO, typ: Type::I32, name: "MAXIMUM_APPARENT_POWER_FROM_GRID" };

        pub const STATE_1:                          Register = Register { address: 32000, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::BF , name: "STATE_1"                          };
        pub const STATE_2:                          Register = Register { address: 32002, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::BF , name: "STATE_2"                          };
        pub const STATE_3:                          Register = Register { address: 32003, quantity:  2, gain: 0, unit: None        , access: Access::RO, typ: Type::BF , name: "STATE_3"                          };
        pub const ALARM_1:                          Register = Register { address: 32008, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::BF , name: "ALARM_1"                          };
        pub const ALARM_2:                          Register = Register { address: 32009, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::BF , name: "ALARM_2"                          };
        pub const ALARM_3:                          Register = Register { address: 32010, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::BF , name: "ALARM_3"                          };

        pub const PV1_VOLTAGE:                      Register = Register { address: 32016, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::I16, name: "PV1_VOLTAGE"                      };
        pub const PV1_CURRENT:                      Register = Register { address: 32017, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: Type::I16, name: "PV1_CURRENT"                      };
        pub const PV2_VOLTAGE:                      Register = Register { address: 32018, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::I16, name: "PV2_VOLTAGE"                      };
        pub const PV2_CURRENT:                      Register = Register { address: 32019, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: Type::I16, name: "PV2_CURRENT"                      };
        pub const PV3_VOLTAGE:                      Register = Register { address: 32020, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::I16, name: "PV3_VOLTAGE"                      };
        pub const PV3_CURRENT:                      Register = Register { address: 32021, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: Type::I16, name: "PV3_CURRENT"                      };
        pub const PV4_VOLTAGE:                      Register = Register { address: 32022, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::I16, name: "PV4_VOLTAGE"                      };
        pub const PV4_CURRENT:                      Register = Register { address: 32023, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: Type::I16, name: "PV4_CURRENT"                      };

        pub const INPUT_POWER:                      Register = Register { address: 32064, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: Type::I32, name: "INPUT_POWER"                      };

        pub const LINE_VOLTAGE_A_B:                 Register = Register { address: 32066, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::U16, name: "LINE_VOLTAGE_A_B"                 };
        pub const LINE_VOLTAGE_B_C:                 Register = Register { address: 32067, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::U16, name: "LINE_VOLTAGE_B_C"                 };
        pub const LINE_VOLTAGE_C_A:                 Register = Register { address: 32068, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::U16, name: "LINE_VOLTAGE_C_A"                 };
        pub const PHASE_VOLTAGE_A:                  Register = Register { address: 32069, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::U16, name: "PHASE_VOLTAGE_A"                  };
        pub const PHASE_VOLTAGE_B:                  Register = Register { address: 32070, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::U16, name: "PHASE_VOLTAGE_B"                  };
        pub const PHASE_VOLTAGE_C:                  Register = Register { address: 32071, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: Type::U16, name: "PHASE_VOLTAGE_C"                  };
        pub const PHASE_CURRENT_A:                  Register = Register { address: 32072, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO, typ: Type::I32, name: "PHASE_CURRENT_A"                  };
        pub const PHASE_CURRENT_B:                  Register = Register { address: 32074, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO, typ: Type::I32, name: "PHASE_CURRENT_B"                  };
        pub const PHASE_CURRENT_C:                  Register = Register { address: 32076, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO, typ: Type::I32, name: "PHASE_CURRENT_C"                  };

        pub const PEAK_ACTIVE_POWER_DAY:            Register = Register { address: 32078, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: Type::I32, name: "PEAK_ACTIVE_POWER_DAY"            };
        pub const ACTIVE_POWER:                     Register = Register { address: 32080, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: Type::I32, name: "ACTIVE_POWER"                     };
        pub const REACTIVE_POWER:                   Register = Register { address: 32082, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO, typ: Type::I32, name: "REACTIVE_POWER"                   };

        pub const POWER_FACTOR:                     Register = Register { address: 32084, quantity:  1, gain: 3, unit: None        , access: Access::RO, typ: Type::I16, name: "POWER_FACTOR"                     };
        pub const GRID_FREQUENCY:                   Register = Register { address: 32085, quantity:  1, gain: 2, unit: Some("Hz")  , access: Access::RO, typ: Type::U16, name: "GRID_FREQUENCY"                   };
        pub const EFFICIENCY:                       Register = Register { address: 32086, quantity:  1, gain: 2, unit: Some("%")   , access: Access::RO, typ: Type::U16, name: "EFFICIENCY"                       };
        pub const INTERNAL_TEMPERATURE:             Register = Register { address: 32087, quantity:  1, gain: 1, unit: Some("°C")  , access: Access::RO, typ: Type::I16, name: "INTERNAL_TEMPERATURE"             };
        pub const INSULATION_RESISTANCE:            Register = Register { address: 32088, quantity:  1, gain: 3, unit: Some("MΩ")  , access: Access::RO, typ: Type::U16, name: "INSULATION_RESISTANCE"            };

        pub const DEVICE_STATUS:                    Register = Register { address: 32089, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::U16, name: "DEVICE_STATUS"                    };
        pub const fn device_status_to_string(status: u16) -> Option<&'static str> {
            match status {
                0x0000 => Some("Standby: initializing"),
                0x0001 => Some("Standby: detecting insulation resistance"),
                0x0002 => Some("Standby: detecting irradiation"),
                0x0003 => Some("Standby: drid detecting"),
                0x0100 => Some("Starting"),
                0x0200 => Some("On-grid (Off-grid mode: running)"),
                0x0201 => Some("Grid connection: power limited (Off-grid mode: running: power limited)"),
                0x0202 => Some("Grid connection: self- derating (Off-grid mode: running: self-derating)"),
                0x0300 => Some("Shutdown: fault"),
                0x0301 => Some("Shutdown: command"),
                0x0302 => Some("Shutdown: OVGR"),
                0x0303 => Some("Shutdown: communication disconnected"),
                0x0304 => Some("Shutdown: power limited"),
                0x0305 => Some("Shutdown: manual startup required"),
                0x0306 => Some("Shutdown: DC switches disconnected"),
                0x0307 => Some("Shutdown: rapid cutoff"),
                0x0308 => Some("Shutdown: input underpower"),
                0x0401 => Some("Grid scheduling: cosφ-P curve"),
                0x0402 => Some("Grid scheduling: Q-U curve"),
                0x0403 => Some("Grid scheduling: PF-U curve"),
                0x0404 => Some("Grid scheduling: dry contact"),
                0x0405 => Some("Grid scheduling: Q-P curve"),
                0x0500 => Some("Spot-check ready"),
                0x0501 => Some("Spot-checking"),
                0x0600 => Some("Inspecting"),
                0x0700 => Some("AFCI self check"),
                0x0800 => Some("I-V scanning"),
                0x0900 => Some("DC input detection"),
                0x0A00 => Some("Running: off-grid charging"),
                0xA000 => Some("Standby: no irradiation"),
                _ => None
            }
        } 
        
            

        pub const FAULT_CODE:                       Register = Register { address: 32090, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::U16, name: "FAULT_CODE"                       };
        pub const STARTUP_TIME:                     Register = Register { address: 32091, quantity:  2, gain: 0, unit: None        , access: Access::RO, typ: Type::U32, name: "STARTUP_TIME"                     };
        pub const SHUTDOWN_TIME:                    Register = Register { address: 32093, quantity:  2, gain: 0, unit: None        , access: Access::RO, typ: Type::U32, name: "SHUTDOWN_TIME"                    };
        pub const ACC_ENERGY_YIELD:                 Register = Register { address: 32106, quantity:  2, gain: 2, unit: Some("kWh") , access: Access::RO, typ: Type::U32, name: "ACC_ENERGY_YIELD"                 };
        pub const ENERGY_YIELD_DAY:                 Register = Register { address: 32114, quantity:  2, gain: 2, unit: Some("kWh") , access: Access::RO, typ: Type::U32, name: "ENERGY_YIELD_DAY"                 };

        pub mod storage {
            use crate::registers::{Access, Register, Type};

            pub const RUNNING_STATUS:               Register = Register { address: 37000, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: Type::U16, name: "RUNNING_STATUS"                   };
            pub const fn running_status_to_string(status: u16) -> Option<&'static str> {
                match status {
                    0 => Some("offline"),
                    1 => Some("standby"),
                    2 => Some("running"),
                    3 => Some("fault"),
                    4 => Some("sleep mode"),
                    _ => None
                }
            }
            pub const CHARGE_DISCHARGE_POWER:       Register = Register { address: 37001, quantity:  2, gain: 0, unit: Some("W")   , access: Access::RO, typ: Type::I32, name: "CHARGE_DISCHARGE_POWER"           };
            pub const CHARGE_CAPACITY_DAY:          Register = Register { address: 37015, quantity:  2, gain: 2, unit: Some("kWh") , access: Access::RO, typ: Type::U32, name: "CHARGE_CAPACITY_DAY"              };
            pub const DISCHARGE_CAPACITY_DAY:       Register = Register { address: 37017, quantity:  2, gain: 2, unit: Some("kWh") , access: Access::RO, typ: Type::U32, name: "DISCHARGE_CAPACITY_DAY"           };
            // pub const ACTIVE_POWER:                 Register = Register { address: 37113, quantity:  2, gain: 0, unit: Some("W")   , access: Access::RO, typ: Type::U16, name: "INSULATION_RESISTANCE"            };
        }
        // =======================================
        // ===== START OF READ-WRITE SECTION =====
        // =======================================

        pub const STARTUP:                          Register = Register { address: 40200, quantity:  1, gain: 0, unit: None        , access: Access::WO, typ: Type::U16, name: "STARTUP"                          };
        pub const SHUTDOWN:                         Register = Register { address: 40201, quantity:  1, gain: 0, unit: None        , access: Access::WO, typ: Type::U16, name: "SHUTDOWN"                         };
        pub const GRID_CODE:                        Register = Register { address: 42000, quantity:  1, gain: 0, unit: None        , access: Access::RW, typ: Type::U16, name: "GRID_CODE"                        };
        
        pub const TIME_ZONE:                        Register = Register { address: 43006, quantity:  1, gain: 0, unit: Some("min") , access: Access::RW, typ: Type::I16, name: "TIME_ZONE"                        };

    }
}

// connect_timeout: "5s"
// read_timeout: "5s"
// write_timeout: "5s"
// host: "192.168.200.1"
// port: 6607

enum Client {
    TCP(Transport),
}
pub struct Inverter {
    client: Client,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Modbus(modbus::Error),
    Conversion,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(io_err) => io_err.fmt(f),
            Error::Modbus(mb_err) => mb_err.fmt(f),
            Error::Conversion => write!(f, "Failed to convert Vec<u16> result."),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(io_err) => Some(io_err),
            Error::Modbus(mb_err) => Some(mb_err),
            Error::Conversion => None,
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}
impl From<modbus::Error> for Error {
    fn from(err: modbus::Error) -> Error {
        Error::Modbus(err)
    }
}
///
/// # Examples
/// ```
/// # use std::time::Duration;
/// use huawei_solar::Inverter;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Inverter::connect_tcp(
///     Some("192.168.200.1"),
///     Some(6607),
///     Some(0),
///     Some(Duration::from_secs(5)),
///     Some(Duration::from_secs(5)),
///     Some(Duration::from_secs(5))
/// );
/// # Ok(())
/// # }
/// ```
impl Inverter {
    /// Connect to Inverter directly over TCP
    ///
    /// Default values are taken from an
    ///
    pub fn connect_tcp(
        addr: Option<&str>,
        port: Option<u16>,
        modbus_uid: Option<u8>,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
        connect_timeout: Option<Duration>,
    ) -> Result<Self, Error> {
        let mb_client = modbus::tcp::Transport::new_with_cfg(
            addr.unwrap_or("192.168.200.1"),
            modbus::Config {
                modbus_uid: modbus_uid.unwrap_or(0),
                tcp_port: port.unwrap_or(6607),
                tcp_read_timeout: read_timeout.or(Some(Duration::from_secs(5))),
                tcp_write_timeout: write_timeout.or(Some(Duration::from_secs(5))),
                tcp_connect_timeout: connect_timeout.or(Some(Duration::from_secs(5))),
            },
        )?;
        Ok(Inverter {
            client: Client::TCP(mb_client),
        })
    }

    // pub fn read<T: RegisterType>(self, reg: registers::Register<T>) -> Result<T, Error> {
    //     let value = match self.client {
    //         Client::TCP(mut tcp_client) => {
    //             tcp_client.read_holding_registers(reg.address, reg.quantity.into())?
    //         }
    //     };
    //     T::from(value).ok_or(Error::Conversion)
    // }
    pub fn read_raw(&mut self, reg: registers::Register) -> Result<Vec<u16>, Error> {
        let value = match self.client {
            Client::TCP(ref mut tcp_client) => modbus::Client::read_holding_registers(
                tcp_client,
                reg.address,
                reg.quantity.into(),
            )?,
        };
        Ok(value)
    }

    pub fn read_batch_raw(
        &mut self,
        regs: &[&registers::Register],
    ) -> Result<Vec<Vec<u16>>, modbus::Error> {
        if regs.is_empty() {
            return Ok(Vec::new());
        }
        let min = regs.iter().map(|r| r.address).min().unwrap();
        // actually highest read address +1
        let max = regs
            .iter()
            .map(|r| r.address + r.quantity as u16)
            .max()
            .unwrap();

        let values = match self.client {
            Client::TCP(ref mut tcp_client) => {
                modbus::Client::read_holding_registers(tcp_client, min, max - min)?
            }
        };

        let chunked = regs
            .iter()
            .map(|reg| {
                let start = (reg.address - min) as usize;
                let size = reg.quantity as usize;
                values[start..start + size].to_vec()
            })
            .collect();

        Ok(chunked)
    }

    pub fn read_batch<'a, 'b>(
        &mut self,
        regs: &'a [&registers::Register<'b>],
        // fun: fn(&registers::Register, registers::RegValue),
    ) -> Result<Vec<registers::RegValue<'a, 'b>>, modbus::Error> {
        let values = Inverter::read_batch_raw(self, regs)?;

        values
            .into_iter()
            .zip(regs)
            .map(|(val, reg)| -> Result<registers::RegValue, modbus::Error> {
                Ok(registers::RegValue {
                    reg: reg,
                    val: reg
                        .typ
                        .convert(&val)
                        .ok_or(modbus::Error::InvalidResponse)?,
                })
            })
            .collect()
    }
    pub fn read_batch_retry<'a, 'b>(
        &mut self,
        regs: &'a [&registers::Register<'b>],
        retries: u8,
    ) -> Result<Vec<registers::RegValue<'a, 'b>>, modbus::Error> {
        match Inverter::read_batch(self, regs) {
            Ok(v) => Ok(v),
            Err(e) => {
                if retries > 0 {
                    sleep(Duration::from_millis(200));
                    Inverter::read_batch_retry(self, regs, retries - 1)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn disconnect(&mut self) -> Result<(), modbus::Error> {
        match self.client {
            Client::TCP(ref mut tcp_client) => modbus::Transport::close(tcp_client),
        }
    }
}

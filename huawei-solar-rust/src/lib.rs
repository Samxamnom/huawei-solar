use std::time::Duration;

use modbus::Transport;

// use registers::RegisterType;

pub mod registers {
    use bit_vec::BitVec;
    pub trait RegisterType {
        fn convert(vec: Vec<u16>) -> Option<Self>
        where
            Self: Sized;
    }
    impl RegisterType for String {
        fn convert(vec: Vec<u16>) -> Option<Self> {
            String::from_utf8(vec.into_iter().flat_map(|u| u.to_be_bytes()).collect()).ok()
        }
    }
    impl RegisterType for u16 {
        fn convert(vec: Vec<u16>) -> Option<Self> {
            vec.first().copied()
        }
    }
    #[test]
    fn convert_u16() {
        assert_eq!(u16::convert(vec![u16::MIN]), Some(u16::MIN));
        assert_eq!(u16::convert(vec![0u16]), Some(0u16));
        assert_eq!(u16::convert(vec![1u16]), Some(1u16));
        assert_eq!(u16::convert(vec![u16::MAX]), Some(u16::MAX));
    }
    impl RegisterType for u32 {
        fn convert(vec: Vec<u16>) -> Option<Self> {
            Some((vec[0] as u32) << 16 | vec[1] as u32)
        }
    }
    #[test]
    fn convert_u32() {
        assert_eq!(u32::convert(vec![u16::MIN, u16::MIN]), Some(u32::MIN));
        assert_eq!(u32::convert(vec![0u16, 0u16]), Some(0u32));
        assert_eq!(
            u32::convert(vec![0x000Fu16, 0xF0FFu16]),
            Some(0x000FF0FFu32)
        );
        assert_eq!(u32::convert(vec![u16::MAX, u16::MAX]), Some(u32::MAX));
    }

    impl RegisterType for i16 {
        fn convert(vec: Vec<u16>) -> Option<Self> {
            vec.first().map(|v| *v as i16)
        }
    }

    #[test]
    fn convert_i16() {
        assert_eq!(i16::convert(vec![0u16]), Some(0i16));
        assert_eq!(i16::convert(vec![1u16]), Some(1i16));
        assert_eq!(i16::convert(vec![u16::MAX]), Some(-1i16));
    }

    impl RegisterType for i32 {
        fn convert(vec: Vec<u16>) -> Option<Self> {
            Some((vec[0] as i32) << 16 | vec[1] as i32)
        }
    }
    #[test]
    fn convert_i32() {
        assert_eq!(<i32 as RegisterType>::convert(vec![0u16, 0u16]), Some(0i32));
        assert_eq!(
            <i32 as RegisterType>::convert(vec![0x000Fu16, 0xF0FFu16]),
            Some(0x000FF0FFi32)
        );
        assert_eq!(
            <i32 as RegisterType>::convert(vec![u16::MAX, u16::MAX]),
            Some(-1i32)
        );
    }

    // TODO
    // BitVec read bits from MSB to LSB -- probably wrong direction
    impl RegisterType for BitVec {
        fn convert(vec: Vec<u16>) -> Option<Self> {
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
        assert!(BitVec::convert(vec![0u16, 0u16]).unwrap().capacity() >= 32);
        assert_eq!(BitVec::convert(vec![0u16, 0u16]).unwrap()[0], false);
        assert_eq!(BitVec::convert(vec![8u16, 0u16]).unwrap()[12], true);
    }

    #[derive(Debug)]
    pub enum RegType {
        U16,
        U32,
        I16,
        I32,
        BF,
        STR,
    }
    impl RegType {
        #[rustfmt::skip]
        pub fn convert(&self, val: Vec<u16>) -> Option<RegValue> {
            match self {
                RegType::U16 => Some(RegValue::U16(   u16::convert(val)?)),
                RegType::U32 => Some(RegValue::U32(   u32::convert(val)?)),
                RegType::I16 => Some(RegValue::I16(   i16::convert(val)?)),
                RegType::I32 => Some(RegValue::I32(   i32::convert(val)?)),
                RegType::BF  => Some(RegValue::BF (BitVec::convert(val)?)),
                RegType::STR => Some(RegValue::STR(String::convert(val)?)),
            }
        }
    }

    #[derive(Debug)]
    pub enum RegValue {
        U16(u16),
        U32(u32),
        I16(i16),
        I32(i32),
        BF(BitVec),
        STR(String),
    }

    pub enum Access {
        RO,
        WO,
        RW,
    }

    pub struct Register<'a /* , T: RegisterType */> {
        pub address: u16,
        pub quantity: u8,
        pub gain: u8,
        pub unit: Option<&'a str>,
        pub access: Access,
        pub typ: RegType,
        pub name: &'a str,
        // typ: std::marker::PhantomData<T>,
    }

    pub use nofmt::*;
    #[rustfmt::skip]
    mod nofmt {
        // use bit_vec::BitVec;
        use super::{Access, RegType, Register};

        pub const MODEL:                            Register = Register { address: 30000, quantity: 15, gain: 0, unit: None        , access: Access::RO, typ: RegType::STR, name: "MODEL"                            };
        pub const SN:                               Register = Register { address: 30015, quantity: 10, gain: 0, unit: None        , access: Access::RO, typ: RegType::STR, name: "SN"                               };
        pub const PN:                               Register = Register { address: 30025, quantity: 10, gain: 0, unit: None        , access: Access::RO, typ: RegType::STR, name: "PN"                               };
        pub const MODEL_ID:                         Register = Register { address: 30070, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::U16, name: "MODEL_ID"                         };
        pub const NUMBER_OF_PV_STRINGS:             Register = Register { address: 30071, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::U16, name: "NUMBER_OF_PV_STRINGS"             };
        pub const NUMBER_OF_MPP_TRACKERS:           Register = Register { address: 30072, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::U16, name: "NUMBER_OF_MPP_TRACKERS"           };
        pub const RATED_POWER:                      Register = Register { address: 30073, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: RegType::U32, name: "RATED_POWER"                      };
        pub const MAXIMUM_ACTIVE_POWER:             Register = Register { address: 30075, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: RegType::U32, name: "MAXIMUM_ACTIVE_POWER"             };
        pub const MAXIMUM_APPARENT_POWER:           Register = Register { address: 30077, quantity:  2, gain: 3, unit: Some("kVA") , access: Access::RO, typ: RegType::U32, name: "MAXIMUM_APPARENT_POWER"           };
        pub const MAXIMUM_REACTIVE_POWER_TO_GRID:   Register = Register { address: 30079, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO, typ: RegType::I32, name: "MAXIMUM_REACTIVE_POWER_TO_GRID"   };
        pub const MAXIMUM_APPARENT_POWER_FROM_GRID: Register = Register { address: 30081, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO, typ: RegType::I32, name: "MAXIMUM_APPARENT_POWER_FROM_GRID" };

        pub const STATE_1:                          Register = Register { address: 32000, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::BF , name: "STATE_1"                          };
        pub const STATE_2:                          Register = Register { address: 32002, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::BF , name: "STATE_2"                          };
        pub const STATE_3:                          Register = Register { address: 32003, quantity:  2, gain: 0, unit: None        , access: Access::RO, typ: RegType::BF , name: "STATE_3"                          };
        pub const ALARM_1:                          Register = Register { address: 32008, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::BF , name: "ALARM_1"                          };
        pub const ALARM_2:                          Register = Register { address: 32009, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::BF , name: "ALARM_2"                          };
        pub const ALARM_3:                          Register = Register { address: 32010, quantity:  1, gain: 0, unit: None        , access: Access::RO, typ: RegType::BF , name: "ALARM_3"                          };

        pub const PV1_VOLTAGE:                      Register = Register { address: 32016, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::I16, name: "PV1_VOLTAGE"                      };
        pub const PV1_CURRENT:                      Register = Register { address: 32017, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: RegType::I16, name: "PV1_CURRENT"                      };
        pub const PV2_VOLTAGE:                      Register = Register { address: 32018, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::I16, name: "PV2_VOLTAGE"                      };
        pub const PV2_CURRENT:                      Register = Register { address: 32019, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: RegType::I16, name: "PV2_CURRENT"                      };
        pub const PV3_VOLTAGE:                      Register = Register { address: 32020, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::I16, name: "PV3_VOLTAGE"                      };
        pub const PV3_CURRENT:                      Register = Register { address: 32021, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: RegType::I16, name: "PV3_CURRENT"                      };
        pub const PV4_VOLTAGE:                      Register = Register { address: 32022, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::I16, name: "PV4_VOLTAGE"                      };
        pub const PV4_CURRENT:                      Register = Register { address: 32023, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO, typ: RegType::I16, name: "PV4_CURRENT"                      };

        pub const INPUT_POWER:                      Register = Register { address: 32064, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: RegType::I32, name: "INPUT_POWER"                      };

        pub const LINE_VOLTAGE_A_B:                 Register = Register { address: 32066, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::U16, name: "LINE_VOLTAGE_A_B"                 };
        pub const LINE_VOLTAGE_B_C:                 Register = Register { address: 32067, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::U16, name: "LINE_VOLTAGE_B_C"                 };
        pub const LINE_VOLTAGE_C_A:                 Register = Register { address: 32068, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::U16, name: "LINE_VOLTAGE_C_A"                 };
        pub const PHASE_VOLTAGE_A:                  Register = Register { address: 32069, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::U16, name: "PHASE_VOLTAGE_A"                  };
        pub const PHASE_VOLTAGE_B:                  Register = Register { address: 32070, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::U16, name: "PHASE_VOLTAGE_B"                  };
        pub const PHASE_VOLTAGE_C:                  Register = Register { address: 32071, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO, typ: RegType::U16, name: "PHASE_VOLTAGE_C"                  };
        pub const PHASE_CURRENT_A:                  Register = Register { address: 32072, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO, typ: RegType::I32, name: "PHASE_CURRENT_A"                  };
        pub const PHASE_CURRENT_B:                  Register = Register { address: 32074, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO, typ: RegType::I32, name: "PHASE_CURRENT_B"                  };
        pub const PHASE_CURRENT_C:                  Register = Register { address: 32076, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO, typ: RegType::I32, name: "PHASE_CURRENT_C"                  };

        pub const PEAK_ACTIVE_POWER_DAY:            Register = Register { address: 32078, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: RegType::I32, name: "PEAK_ACTIVE_POWER_DAY"            };
        pub const ACTIVE_POWER:                     Register = Register { address: 32080, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO, typ: RegType::I32, name: "ACTIVE_POWER"                     };
        pub const REACTIVE_POWER:                   Register = Register { address: 32082, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO, typ: RegType::I32, name: "REACTIVE_POWER"                   };

        pub const POWER_FACTOR:                     Register = Register { address: 32084, quantity:  1, gain: 3, unit: None        , access: Access::RO, typ: RegType::I16, name: "POWER_FACTOR"                     };
        pub const GRID_FREQUENCY:                   Register = Register { address: 32085, quantity:  1, gain: 2, unit: Some("Hz")  , access: Access::RO, typ: RegType::U16, name: "GRID_FREQUENCY"                   };
        pub const EFFICIENCY:                       Register = Register { address: 32086, quantity:  1, gain: 2, unit: Some("%")   , access: Access::RO, typ: RegType::U16, name: "EFFICIENCY"                       };
        pub const INTERNAL_TEMPERATURE:             Register = Register { address: 32087, quantity:  1, gain: 1, unit: Some("°C")  , access: Access::RO, typ: RegType::I16, name: "INTERNAL_TEMPERATURE"             };
        pub const INSULATION_RESISTANCE:            Register = Register { address: 32088, quantity:  1, gain: 3, unit: Some("MΩ")  , access: Access::RO, typ: RegType::U16, name: "INSULATION_RESISTANCE"            };

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
    pub fn read(&mut self, reg: registers::Register) -> Result<Vec<u16>, Error> {
        let value = match self.client {
            Client::TCP(ref mut tcp_client) => modbus::Client::read_holding_registers(
                tcp_client,
                reg.address,
                reg.quantity.into(),
            )?,
        };
        Ok(value)
    }

    pub fn read_batch(
        &mut self,
        regs: &[registers::Register],
        // fun: fn(&registers::Register, registers::RegValue),
    ) -> Result<Vec<registers::RegValue>, modbus::Error> {
        let values = Inverter::read_batch_raw(self, regs)?;
        values
            .into_iter()
            .zip(regs)
            .map(|(val, reg)| -> Result<registers::RegValue, modbus::Error> {
                Ok(reg.typ.convert(val).ok_or(modbus::Error::InvalidResponse)?)
            })
            .collect()
    }

    pub fn read_batch_raw(
        &mut self,
        regs: &[registers::Register],
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

        println!("min max: {} {}", min, max);
        let values = match self.client {
            Client::TCP(ref mut tcp_client) => {
                modbus::Client::read_holding_registers(tcp_client, min, max - min)?
            }
        };
        println!("values: {:?}", values);

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

    pub fn disconnect(&mut self) -> Result<(), modbus::Error> {
        match self.client {
            Client::TCP(ref mut tcp_client) => modbus::Transport::close(tcp_client),
        }
    }
    // match mb_client.read_holding_registers(start_address, end_address - start_address) {
    //     Ok(res) => {
    //         println!(
    //             "Read regs({}..{}): {:?}",
    //             start_address,
    //             end_address - start_address,
    //             res
    //         );
    //         for i in 0..res.len() {
    //             values.insert(start_address + i as u16, res[i]);
    //         }
    //     }
    //     Err(err) => {
    //         eprintln!("Modbus error: {}", err.to_string());
    //         continue 'out;
    //     }
    // }

    //     let mut last_read = DateTime::<Utc>::MIN_UTC;
    //     'out: loop {
    //         // find next query job and time of it
    //         let mut next_job = DateTime::<Utc>::MAX_UTC;
    //         let mut tables = vec![];
    //         for table in &cfg.queries {
    //             let table_time = *table
    //                 .cron
    //                 .upcoming(Utc)
    //                 .next()
    //                 .get_or_insert(DateTime::<Utc>::MAX_UTC);
    //             if next_job > table_time {
    //                 next_job = table_time;
    //                 tables.clear();
    //             }
    //             if table_time == next_job {
    //                 tables.push(table);
    //             }
    //         }
    //         // sleep until next invoke
    //         sleep((next_job - Utc::now()).to_std().unwrap_or(Duration::ZERO));

    //         // query modbus slave
    //         let mut regs = vec![];
    //         tables.iter_mut().for_each(|q| regs.extend(&q.values));
    //         regs.sort_by_key(|a| a.address);

    //         let mut values = HashMap::new();
    //         // group registers to single request
    //         while !regs.is_empty() {
    //             let reg = regs.remove(0);
    //             let start_address = reg.address;
    //             let mut end_address = start_address + reg.data_type.size();

    //             while !regs.is_empty() {
    //                 let reg = regs[0];
    //                 if reg.address == end_address {
    //                     end_address += reg.data_type.size();
    //                     regs.remove(0);
    //                 } else {
    //                     break;
    //                 }
    //             }

    //             // request whole group
    //             println!(
    //                 "since last read: {}, sleeping: {:?}",
    //                 Utc::now() - last_read,
    //                 (chrono::Duration::from_std(Duration::from_millis(500))?
    //                     - (Utc::now() - last_read))
    //                     .to_std()
    //                     .unwrap_or(Duration::ZERO)
    //             );
    //             sleep(
    //                 (chrono::Duration::from_std(Duration::from_millis(500))?
    //                     - (Utc::now() - last_read))
    //                     .to_std()
    //                     .unwrap_or(Duration::ZERO),
    //             );
    //             last_read = Utc::now();
    //             match mb_client.read_holding_registers(start_address, end_address - start_address) {
    //                 Ok(res) => {
    //                     println!(
    //                         "Read regs({}..{}): {:?}",
    //                         start_address,
    //                         end_address - start_address,
    //                         res
    //                     );
    //                     for i in 0..res.len() {
    //                         values.insert(start_address + i as u16, res[i]);
    //                     }
    //                 }
    //                 Err(err) => {
    //                     eprintln!("Modbus error: {}", err.to_string());
    //                     continue 'out;
    //                 }
    //             }
    //         }

    //         // convert and write to db
    //         /* type.convert((r.address..r.address + r.data_type.size()).map(|addr| *values.get(&addr).unwrap()).collect(), r.scale)*/
    //         for table in tables {
    //             // params.extend(table.values.iter().map(|r| r.data_type.convert((r.address..r.address+r.data_type.size()).map(|addr| *values.get(&addr).unwrap()), r.scale) /*as (dyn ToSql + Sync)*/));
    //             match db_client.execute(
    //                 &format!(
    //                     "INSERT INTO {} ({}) VALUES ({})",
    //                     &table.table,
    //                     table
    //                         .values
    //                         .iter()
    //                         .map(|elem| &elem.name)
    //                         .fold(String::from("time"), |accu, ele| accu + "," + ele),
    //                     table
    //                         .values
    //                         .iter()
    //                         .map(|r| r.data_type.convert(
    //                             (r.address..r.address + r.data_type.size())
    //                                 .map(|addr| *values.get(&addr).unwrap())
    //                                 .collect(),
    //                             r.scale
    //                         ))
    //                         .fold(
    //                             format!("'{}'", next_job.with_timezone(&Local).to_string()),
    //                             |accu, ele| accu + "," + &ele.to_string()
    //                         ) // (2..table.values.len() + 2).fold(String::from("$1"), |accu, ele| accu + ", $" + &ele.to_string())
    //                 ),
    //                 // &vec![&5f32 as &(dyn ToSql + Sync)][..]
    //                 // &table.values.iter().map(|r| &(values.get(&r.address).unwrap_or(&0u16) as &f32) as &(dyn ToSql + Sync)).collect::<Vec<&(dyn ToSql + Sync)>>()[..],
    //                 &[],
    //             ) {
    //                 Ok(_) => {}
    //                 Err(err) => eprintln!("Database error: {}", err.to_string()),
    //             }
    //         }
    //     }
    // }
}

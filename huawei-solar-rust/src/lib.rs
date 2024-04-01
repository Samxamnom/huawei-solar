use std::time::Duration;

use modbus::Transport;

// use registers::RegisterType;

pub mod registers {
    use bit_vec::BitVec;
    pub trait RegisterType {
        fn from(vec: Vec<u16>) -> Option<Self>
        where
            Self: Sized;
    }
    impl RegisterType for String {
        fn from(vec: Vec<u16>) -> Option<Self> {
            String::from_utf8(vec.into_iter().flat_map(|u| u.to_be_bytes()).collect()).ok()
        }
    }
    impl RegisterType for u16 {
        fn from(vec: Vec<u16>) -> Option<Self> {
            vec.first().copied()
        }
    }
    #[test]
    fn convert_u16() {
        assert_eq!(<u16 as RegisterType>::from(vec![u16::MIN]), Some(u16::MIN));
        assert_eq!(<u16 as RegisterType>::from(vec![0u16]), Some(0u16));
        assert_eq!(<u16 as RegisterType>::from(vec![1u16]), Some(1u16));
        assert_eq!(<u16 as RegisterType>::from(vec![u16::MAX]), Some(u16::MAX));
    }
    impl RegisterType for u32 {
        fn from(vec: Vec<u16>) -> Option<Self> {
            Some((vec[0] as u32) << 16 | vec[1] as u32)
        }
    }
    #[test]
    fn convert_u32() {
        assert_eq!(
            <u32 as RegisterType>::from(vec![u16::MIN, u16::MIN]),
            Some(u32::MIN)
        );
        assert_eq!(<u32 as RegisterType>::from(vec![0u16, 0u16]), Some(0u32));
        assert_eq!(
            <u32 as RegisterType>::from(vec![0x000Fu16, 0xF0FFu16]),
            Some(0x000FF0FFu32)
        );
        assert_eq!(
            <u32 as RegisterType>::from(vec![u16::MAX, u16::MAX]),
            Some(u32::MAX)
        );
    }

    impl RegisterType for i16 {
        fn from(vec: Vec<u16>) -> Option<Self> {
            vec.first().map(|v| *v as i16)
        }
    }

    #[test]
    fn convert_i16() {
        assert_eq!(<i16 as RegisterType>::from(vec![0u16]), Some(0i16));
        assert_eq!(<i16 as RegisterType>::from(vec![1u16]), Some(1i16));
        assert_eq!(<i16 as RegisterType>::from(vec![u16::MAX]), Some(-1i16));
    }

    impl RegisterType for i32 {
        fn from(vec: Vec<u16>) -> Option<Self> {
            Some((vec[0] as i32) << 16 | vec[1] as i32)
        }
    }
    #[test]
    fn convert_i32() {
        assert_eq!(<i32 as RegisterType>::from(vec![0u16, 0u16]), Some(0i32));
        assert_eq!(
            <i32 as RegisterType>::from(vec![0x000Fu16, 0xF0FFu16]),
            Some(0x000FF0FFi32)
        );
        assert_eq!(
            <i32 as RegisterType>::from(vec![u16::MAX, u16::MAX]),
            Some(-1i32)
        );
    }

    // TODO
    // BitVec read bits from MSB to LSB -- probably wrong direction
    impl RegisterType for BitVec {
        fn from(vec: Vec<u16>) -> Option<Self> {
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
        assert!(<BitVec as RegisterType>::from(vec![0u16, 0u16]).unwrap().capacity() >= 32);
        assert_eq!(<BitVec as RegisterType>::from(vec![0u16, 0u16]).unwrap()[0], false);
        assert_eq!(<BitVec as RegisterType>::from(vec![8u16, 0u16]).unwrap()[12], true);
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
        // typ: std::marker::PhantomData<T>,
    }

    pub use nofmt::*;
    #[rustfmt::skip]
    mod nofmt {
        // use bit_vec::BitVec;
        use super::{Access, Register};

        pub const MODEL:                            Register/* <String> */ = Register/* ::<String> */ { address: 30000, quantity: 15, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const SN:                               Register/* <String> */ = Register/* ::<String> */ { address: 30015, quantity: 10, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PN:                               Register/* <String> */ = Register/* ::<String> */ { address: 30025, quantity: 10, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const MODEL_ID:                         Register/* <u16>    */ = Register/* ::<u16>    */ { address: 30070, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const NUMBER_OF_PV_STRINGS:             Register/* <u16>    */ = Register/* ::<u16>    */ { address: 30071, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const NUMBER_OF_MPP_TRACKERS:           Register/* <u16>    */ = Register/* ::<u16>    */ { address: 30072, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const RATED_POWER:                      Register/* <u32>    */ = Register/* ::<u32>    */ { address: 30073, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const MAXIMUM_ACTIVE_POWER:             Register/* <u32>    */ = Register/* ::<u32>    */ { address: 30075, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const MAXIMUM_APPARENT_POWER:           Register/* <u32>    */ = Register/* ::<u32>    */ { address: 30077, quantity:  2, gain: 3, unit: Some("kVA") , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const MAXIMUM_REACTIVE_POWER_TO_GRID:   Register/* <i32>    */ = Register/* ::<i32>    */ { address: 30079, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const MAXIMUM_APPARENT_POWER_FROM_GRID: Register/* <i32>    */ = Register/* ::<i32>    */ { address: 30081, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO/* , typ: std::marker::PhantomData */ };

        pub const STATE_1:                          Register/* <BitVec> */ = Register/* ::<BitVec> */ { address: 32000, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const STATE_2:                          Register/* <BitVec> */ = Register/* ::<BitVec> */ { address: 32002, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const STATE_3:                          Register/* <BitVec> */ = Register/* ::<BitVec> */ { address: 32003, quantity:  2, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const ALARM_1:                          Register/* <BitVec> */ = Register/* ::<BitVec> */ { address: 32008, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const ALARM_2:                          Register/* <BitVec> */ = Register/* ::<BitVec> */ { address: 32009, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const ALARM_3:                          Register/* <BitVec> */ = Register/* ::<BitVec> */ { address: 32010, quantity:  1, gain: 0, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };

        pub const PV1_VOLTAGE:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32016, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PV1_CURRENT:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32017, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PV2_VOLTAGE:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32018, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PV2_CURRENT:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32019, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PV3_VOLTAGE:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32020, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PV3_CURRENT:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32021, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PV4_VOLTAGE:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32022, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PV4_CURRENT:                      Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32023, quantity:  1, gain: 2, unit: Some("A")   , access: Access::RO/* , typ: std::marker::PhantomData */ };

        pub const INPUT_POWER:                      Register/* <i32>    */ = Register/* ::<i32>    */ { address: 32064, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO/* , typ: std::marker::PhantomData */ };

        pub const LINE_VOLTAGE_A_B:                 Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32066, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const LINE_VOLTAGE_B_C:                 Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32067, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const LINE_VOLTAGE_C_A:                 Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32068, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PHASE_VOLTAGE_A:                  Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32069, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PHASE_VOLTAGE_B:                  Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32070, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PHASE_VOLTAGE_C:                  Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32071, quantity:  1, gain: 1, unit: Some("V")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PHASE_CURRENT_A:                  Register/* <i32>    */ = Register/* ::<i32>    */ { address: 32072, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PHASE_CURRENT_B:                  Register/* <i32>    */ = Register/* ::<i32>    */ { address: 32074, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const PHASE_CURRENT_C:                  Register/* <i32>    */ = Register/* ::<i32>    */ { address: 32076, quantity:  2, gain: 3, unit: Some("A")   , access: Access::RO/* , typ: std::marker::PhantomData */ };

        pub const PEAK_ACTIVE_POWER_DAY:            Register/* <i32>    */ = Register/* ::<i32>    */ { address: 32078, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const ACTIVE_POWER:                     Register/* <i32>    */ = Register/* ::<i32>    */ { address: 32080, quantity:  2, gain: 3, unit: Some("kW")  , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const REACTIVE_POWER:                   Register/* <i32>    */ = Register/* ::<i32>    */ { address: 32082, quantity:  2, gain: 3, unit: Some("kVar"), access: Access::RO/* , typ: std::marker::PhantomData */ };

        pub const POWER_FACTOR:                     Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32084, quantity:  1, gain: 3, unit: None        , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const GRID_FREQUENCY:                   Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32085, quantity:  1, gain: 2, unit: Some("Hz")  , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const EFFICIENCY:                       Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32086, quantity:  1, gain: 2, unit: Some("%")   , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const INTERNAL_TEMPERATURE:             Register/* <i16>    */ = Register/* ::<i16>    */ { address: 32087, quantity:  1, gain: 1, unit: Some("°C")  , access: Access::RO/* , typ: std::marker::PhantomData */ };
        pub const INSULATION_RESISTANCE:            Register/* <u16>    */ = Register/* ::<u16>    */ { address: 32088, quantity:  1, gain: 3, unit: Some("MΩ")  , access: Access::RO/* , typ: std::marker::PhantomData */ };

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
    ) -> Result<Vec<Vec<u16>>, modbus::Error> {
        if regs.is_empty() {
            return Ok(Vec::new());
        }
        let min = regs.iter().map(|r| r.address).min().unwrap();
        // actually highest read address +1
        let max = regs
            .iter()
            .map(|r| r.address + r.quantity as u16)
            .min()
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

// use std::{env, fs};
// use std::collections::HashMap;
// use std::str::FromStr;
// use std::string::String;
// use std::thread::sleep;
// use std::time::Duration;

// use chrono::{DateTime, Local, Utc};
// use cron::Schedule;
// use modbus::{Client, tcp};
// use postgres::NoTls;
// // use postgres::types::ToSql;
// use serde::de::{self, Deserializer};
// use serde::Deserialize;

// #[derive(Deserialize, Debug)]
// struct Config {
//     #[serde(deserialize_with = "duration")]
//     db_timeout: Duration,
//     modbus: ModbusConfig,
//     queries: Vec<Table>,
// }

// struct InverterData {

// }

// #[derive(Deserialize, Debug)]
// struct ModbusConfig {
//     #[serde(deserialize_with = "duration")]
//     read_timeout: Duration,
//     #[serde(deserialize_with = "duration")]
//     connect_timeout: Duration,
//     #[serde(deserialize_with = "duration")]
//     write_timeout: Duration,
//     host: String,
//     port: u16,
// }

// #[derive(Deserialize, Debug)]
// struct Table {
//     table: String,
//     #[serde(deserialize_with = "schedule")]
//     cron: Schedule,
//     values: Vec<Register>,
// }

// #[derive(Deserialize, Debug)]
// struct Register {
//     name: String,
//     address: u16,
//     scale: f32,
//     // unit: String,
//     #[serde(rename = "type")]
//     data_type: DataType,
// }

// #[derive(Deserialize, Debug)]
// enum DataType {
//     I16,
//     I32,
//     U16,
//     U32,
// }

// impl DataType {
//     fn size(&self) -> u16 {
//         match self {
//             DataType::I16 | DataType::U16 => 1,
//             DataType::I32 | DataType::U32 => 2,
//         }
//     }

//     /// This function converts the output of the modbus client
//     /// to the correct value in f32 format
//     fn convert(&self, data: Vec<u16>, scale: f32) -> f32 {
//         scale * match self {
//             DataType::I16 => data[0] as i16 as f32,
//             DataType::I32 => ((data[0] as i32) << 0x10 | data[1] as i32) as f32,
//             DataType::U16 => data[0] as f32,
//             DataType::U32 => ((data[0] as u32) << 0x10 | data[1] as u32) as f32,
//         }
//     }
// }

// #[test]
// fn data_type_convert_test() {
//     assert_eq!(DataType::U16.convert(vec![u16::MAX], 1.0), u16::MAX as f32);
//     assert_eq!(DataType::I16.convert(vec![u16::MAX], 1.0), -1.0);
//     assert_eq!(DataType::U32.convert(vec![0u16,8000u16], 1.0), 8000.0);
//     assert_eq!(DataType::U32.convert(vec![0u16,8000u16], 0.001), 8.0);
//     assert_eq!(DataType::U32.convert(vec![32, 11], 10.0), (((32 << 16) + 11)*10) as f32);
//     assert_eq!(DataType::I32.convert(vec![u16::MAX, u16::MAX - 53], 1.0), -54.0);
//     assert_eq!(DataType::I32.convert(vec![u16::MAX, u16::MAX], 1.0), -1.0);
// }

// /// [Deserialize] a [Schedule] from a [String]
// fn schedule<'de, D>(deserializer: D) -> Result<Schedule, D::Error>
//     where D: Deserializer<'de>
// {
//     let s = String::deserialize(deserializer)?;
//     Schedule::from_str(&s).map_err(de::Error::custom)
// }

// /// [Deserialize] a [Duration] from a [String] using the [parse_duration] crate
// fn duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
//     where D: Deserializer<'de>
// {
//     let s = String::deserialize(deserializer)?;
//     parse_duration::parse(&s).map_err(de::Error::custom)
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let cfg: Config = serde_yaml::from_str(&fs::read_to_string("./resources/config.yaml")?)?;

//     let mut mb_client = tcp::Transport::new_with_cfg(&cfg.modbus.host, modbus::Config {
//         modbus_uid: 0,
//         tcp_port: cfg.modbus.port,
//         tcp_read_timeout: Some(cfg.modbus.read_timeout),
//         tcp_write_timeout: Some(cfg.modbus.write_timeout),
//         tcp_connect_timeout: Some(cfg.modbus.connect_timeout),
//     })?;

//     let mut db_client = connect_database(5, cfg.db_timeout)?;

//     let mut create_queries = Vec::new();
//     for table in &cfg.queries {
//         create_queries.push(format!("CREATE TABLE IF NOT EXISTS {} ({})",
//             &table.table,
//             &table.values.iter().map(|r| format!("{} real", r.name)).fold(String::from("time timestamptz NOT NULL"), |accu, elem| accu + "," + &elem)
//         ));
//     }
//     db_client.batch_execute(&create_queries.join(";"))?;

//     let mut last_read = DateTime::<Utc>::MIN_UTC;
//     'out: loop {
//         // find next query job and time of it
//         let mut next_job = DateTime::<Utc>::MAX_UTC;
//         let mut tables = vec![];
//         for table in &cfg.queries {
//             let table_time = *table.cron.upcoming(Utc).next().get_or_insert(DateTime::<Utc>::MAX_UTC);
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

//             // fill with debug values
//             // for i in start_address..end_address {
//             //     values.insert(i, rand::random::<u16>());
//             // }
//             // request whole group
//             println!("since last read: {}, sleeping: {:?}", Utc::now() - last_read, (chrono::Duration::from_std(Duration::from_millis(500))? - (Utc::now() - last_read)).to_std().unwrap_or(Duration::ZERO));
//             sleep((chrono::Duration::from_std(Duration::from_millis(500))? - (Utc::now() - last_read)).to_std().unwrap_or(Duration::ZERO));
//             last_read = Utc::now();
//             match mb_client.read_holding_registers(start_address, end_address - start_address) {
//                 Ok(res) =>{
//                     println!("Read regs({}..{}): {:?}", start_address, end_address-start_address, res);
//                     for i in 0..res.len() { values.insert(start_address + i as u16, res[i]); }
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
//             match db_client.execute(&format!("INSERT INTO {} ({}) VALUES ({})",
//                 &table.table,
//                 table.values.iter().map(|elem| &elem.name).fold(String::from("time"), |accu, ele| accu + "," + ele),
//                 table.values.iter().map(|r| r.data_type.convert((r.address..r.address+r.data_type.size()).map(|addr| *values.get(&addr).unwrap()).collect(), r.scale))
//                     .fold(format!("'{}'", next_job.with_timezone(&Local).to_string()), |accu, ele| accu + "," + &ele.to_string())
//                 // (2..table.values.len() + 2).fold(String::from("$1"), |accu, ele| accu + ", $" + &ele.to_string())
//             ),
//                 // &vec![&5f32 as &(dyn ToSql + Sync)][..]
//                 // &table.values.iter().map(|r| &(values.get(&r.address).unwrap_or(&0u16) as &f32) as &(dyn ToSql + Sync)).collect::<Vec<&(dyn ToSql + Sync)>>()[..],
//                 &[]
//             ) {
//                 Ok(_) => {}
//                 Err(err) => eprintln!("Database error: {}", err.to_string())
//             }
//         }
//     }
// }

// fn connect_database(retries: u8, delay: Duration) -> Result<postgres::Client, Box<dyn std::error::Error>> {
//     match postgres::Client::connect(&format!("host={} user={} password={} dbname={} connect_timeout=20",
//         env::var("DB_HOST")?, env::var("DB_USER")?,
//         env::var("DB_PASS")?, env::var("DB_NAME")?), NoTls) {
//         Err(e) => {
//             if retries > 0 {
//                 sleep(delay);
//                 connect_database(retries - 1, delay)
//             } else {
//                 Err(Box::new(e))
//             }
//         }
//         Ok(client) => Ok(client),
//     }
// }

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use huawei_solar::registers::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inverter = huawei_solar::Inverter::connect_tcp(
        Some("192.168.200.1"),
        Some(6607),
        Some(0),
        Some(Duration::from_secs(5)),
        Some(Duration::from_secs(5)),
        Some(Duration::from_secs(5)),
    )?;
    println!("Connecting");

    loop {;
        let now = Instant::now();
        let active_pow = inverter.read(ACTIVE_POWER)?;
        let time = now.elapsed();
        println!("active power {:?} time {:?}", active_pow, time);

        sleep(Duration::from_secs(1));

        let now = Instant::now();
        let active_pow = inverter.read_batch(&vec![
            PV1_VOLTAGE,
            PV1_CURRENT,
            PV2_VOLTAGE,
            PV2_CURRENT,
            PV3_VOLTAGE,
            PV3_CURRENT,
            PV4_VOLTAGE,
            PV4_CURRENT,
        ])?;
        let time = now.elapsed();
        println!("batch {:?} time {:?}", active_pow, time);

        sleep(Duration::from_secs(1))
    }

    // Ok(())
}

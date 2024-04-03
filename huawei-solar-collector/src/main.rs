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

use std::{env, thread::sleep, time::Duration};

use chrono::{DateTime, Local, TimeZone};
use huawei_solar::{registers::*, Inverter};
use postgres::NoTls;

#[derive(Debug)]
struct DbTable<'a> {
    name: String,
    values: Vec<(&'a str, Register<'a>)>,
    alignment: Duration,
    next_read: DateTime<Local>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Inverter over TCP");
    let mut inverter = connect_inverter()?;
    println!("Connected!");

    let status = get_status(&mut inverter)?;

    println!();
    println!("Connecting to Timescale database");
    // let mut db_client = connect_database(12, Duration::from_secs(5))?;
    println!("Connected!");

    let mut tables = create_tables(&status);

    println!("Creating DB tables");
    let create_queries = tables
        .iter()
        .map(|table| {
            format!(
                "CREATE TABLE IF NOT EXISTS {} ({})",
                &table.name,
                &table
                    .values
                    .iter()
                    .map(|r| format!("{} real", r.0))
                    .fold(String::from("time timestamptz NOT NULL"), |accu, elem| accu
                        + ","
                        + &elem)
            )
        })
        .collect::<Vec<String>>();
    // db_client.batch_execute(&create_queries.join(";"))?;
    println!("Creation done");

    println!("Collecting Data");
    loop {
        let now = Local::now();
        tables
            .iter_mut()
            .filter(|t| t.next_read < now)
            .try_for_each(|t| -> Result<(), Box<dyn std::error::Error>> {
                // read table
                let regs = t
                    .values
                    .iter()
                    .map(|v| &v.1)
                    .collect::<Vec<&Register<'static>>>();
                let values = inverter.read_batch_retry(&regs, 10)?;
                // db_client.execute(
                //     &format!(
                //         "INSERT INTO {} ({}) VALUES ({})",
                //         t.name,
                //         t.values
                //             .iter()
                //             .fold(String::from("time"), |accu, ele| accu + "," + ele.0),
                //         values
                //             .iter()
                //             .fold(format!("'{}'", Local::now().to_string()), |accu, ele| accu
                //                 + ","
                //                 + &ele.to_float().unwrap().to_string())
                //     ),
                //     &[],
                // )?;
                t.next_read = next_aligned_timepoint(t.alignment);
                println!("next read: {}", t.next_read);

                Ok(())
            })?;

        let next_req = tables.iter().map(|t| t.next_read).min();
        if let None = next_req {
            break;
        }
        let sleep_time = (next_req.unwrap() - Local::now()).to_std();
        if let Ok(dur) = sleep_time {
            println!("sleep time: {:?}", dur);
            sleep(dur);
        }
    }

    inverter.disconnect()?;

    Ok(())
}

fn connect_database(
    retries: u8,
    delay: Duration,
) -> Result<postgres::Client, Box<dyn std::error::Error>> {
    let db_timeout = 10;
    match postgres::Client::connect(
        &format!(
            "host={} user={} password={} dbname={} connect_timeout={}",
            env::var("DB_HOST")?,
            env::var("DB_USER")?,
            env::var("DB_PASS")?,
            env::var("DB_NAME")?,
            db_timeout
        ),
        NoTls,
    ) {
        Ok(client) => Ok(client),
        Err(e) => {
            if retries > 0 {
                sleep(delay);
                connect_database(retries - 1, delay)
            } else {
                Err(Box::new(e))
            }
        }
    }
}

fn connect_inverter() -> Result<Inverter, Box<dyn std::error::Error>> {
    let ip = env::var("INV_ADDR")?;
    let port = env::var("INV_PORT")?.parse::<u16>()?;
    let mb_id = env::var("INV_MBID")?.parse::<u8>()?;

    println!("\tIP: {}", ip);
    println!("\tport: {}", port);
    println!("\tmodbus id: {}", mb_id);

    Ok(huawei_solar::Inverter::connect_tcp(
        Some(&ip),
        Some(port),
        Some(mb_id),
        Some(Duration::from_millis(500)),
        Some(Duration::from_millis(500)),
        Some(Duration::from_millis(500)),
    )?)
}

struct Status {
    strings: u16,
}

fn get_status(inverter: &mut Inverter) -> Result<Status, Box<dyn std::error::Error>> {
    let info_regs = vec![
        &MODEL,
        &SN,
        &PN,
        &MODEL_ID,
        &NUMBER_OF_PV_STRINGS,
        &NUMBER_OF_MPP_TRACKERS,
        &RATED_POWER,
        &MAXIMUM_ACTIVE_POWER,
        &MAXIMUM_APPARENT_POWER,
        &MAXIMUM_REACTIVE_POWER_TO_GRID,
        &MAXIMUM_APPARENT_POWER_FROM_GRID,
    ];
    let info_vals = inverter.read_batch_retry(&info_regs, 10)?;
    let strings = info_vals[4].to_u16()?;

    let info_regs = vec![
        &EFFICIENCY,
        &INTERNAL_TEMPERATURE,
        &DEVICE_STATUS,
        &STARTUP_TIME,
        &SHUTDOWN_TIME,
    ];
    let info_vals2 = inverter.read_batch_retry(&info_regs, 10)?;

    println!("\nInverter");
    println!("\tModel: {} (ID: {})", &info_vals[0], &info_vals[3]);
    println!("\tSN/PN: {}/{}", &info_vals[1], &info_vals[2]);
    if let Value::U16(val) = info_vals2[2].val {
        println!(
            "\tStatus: {}",
            device_status_to_string(val).unwrap_or("invalid")
        );
    }
    if let Value::U32(val) = info_vals2[3].val {
        println!(
            "\tStartup: {:?}",
            Local::timestamp_opt(&Local, val.into(), 0)
                .unwrap()
                .to_string()
        );
    }
    if let Value::U32(val) = info_vals2[4].val {
        if val != u32::MAX {
            println!(
                "\tShutdown: {:?}",
                Local::timestamp_opt(&Local, val.into(), 0)
                    .unwrap()
                    .to_string()
            );
        }
    }
    println!("\tStrings: {}", &info_vals[4],);
    println!("\tTrackers: {}", &info_vals[5],);
    println!("\tMaximum:");
    println!("\t\tactive power  : {}", &info_vals[7]);
    println!("\t\tapparent power: {}", &info_vals[8]);
    println!("\t\treactive power -> grid: {}", &info_vals[9]);
    println!("\t\tapparent power <- grid: {}", &info_vals[10]);
    println!();

    let storage_info_regs = vec![
        &storage::RUNNING_STATUS,
        &storage::CHARGE_DISCHARGE_POWER,
        &storage::CHARGE_CAPACITY_DAY,
        &storage::DISCHARGE_CAPACITY_DAY,
    ];
    let storage_info_vals = inverter.read_batch_retry(&storage_info_regs, 10)?;

    println!("\tStorage:");
    if let Value::U16(val) = storage_info_vals[0].val {
        println!(
            "\t\tStatus: {}",
            storage::running_status_to_string(val).unwrap_or("invalid")
        );
    }
    println!("\t\tcharge/discharge: {}", &storage_info_vals[1]);
    println!("\t\tcharge capacity    (today): {}", &storage_info_vals[2]);
    println!("\t\tdischarge capacity (today): {}", &storage_info_vals[3]);

    Ok(Status { strings })
}

fn create_tables(status: &Status) -> Vec<DbTable<'static>> {
    let mut tables = vec![
        DbTable {
            name: String::from("general"),
            alignment: Duration::from_secs(30),
            values: [
                INPUT_POWER,
                LINE_VOLTAGE_A_B,
                LINE_VOLTAGE_B_C,
                LINE_VOLTAGE_C_A,
                PHASE_CURRENT_A,
                PHASE_CURRENT_B,
                PHASE_CURRENT_C,
                PHASE_VOLTAGE_A,
                PHASE_VOLTAGE_B,
                PHASE_VOLTAGE_C,
                ACTIVE_POWER,
                REACTIVE_POWER,
                // RATED_POWER,
                ACC_ENERGY_YIELD,
                ENERGY_YIELD_DAY,
            ]
            .into_iter()
            .map(|reg| (reg.name, reg))
            .collect(),
            next_read: Local::now(),
        },
        DbTable {
            name: String::from("monitoring"),
            alignment: Duration::from_secs(30),
            values: [EFFICIENCY, INTERNAL_TEMPERATURE]
                .into_iter()
                .map(|reg| (reg.name, reg))
                .collect(),
            next_read: Local::now(),
        },
        DbTable {
            name: String::from("storage"),
            alignment: Duration::from_secs(30),
            values: [
                storage::CHARGE_DISCHARGE_POWER,
                storage::CHARGE_CAPACITY_DAY,
                storage::DISCHARGE_CAPACITY_DAY,
            ]
            .into_iter()
            .map(|reg| (reg.name, reg))
            .collect(),
            next_read: Local::now(),
        },
    ];

    let plant_regs = [
        (PV1_VOLTAGE, PV1_CURRENT),
        (PV2_VOLTAGE, PV2_CURRENT),
        (PV3_VOLTAGE, PV3_CURRENT),
        (PV4_VOLTAGE, PV4_CURRENT),
    ];

    plant_regs
        .into_iter()
        .zip(0..status.strings)
        .for_each(|((volt, curr), i)| {
            tables.push(DbTable {
                alignment: Duration::from_secs(5),
                name: format!("plant_{}", i + 1),
                values: vec![("voltage", volt), ("current", curr)],
                next_read: Local::now(),
            })
        });

    tables
}

fn next_aligned_timepoint(alignment: Duration) -> DateTime<Local> {
    let now = Local::now();
    let nanos = now.timestamp_nanos_opt().unwrap();
    let align = alignment.as_nanos() as i64;

    DateTime::from_timestamp_nanos(nanos - (nanos % align) + align).with_timezone(&Local)
}

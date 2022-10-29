use std::ops::Sub;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use modbus::{Client, Config, Error, tcp};

fn main() -> Result<(), Error>{
    let mut cfg = Config::default();
    cfg.tcp_port = 6607;
    cfg.modbus_uid = 0;
    cfg.tcp_read_timeout = Some(Duration::from_millis(2000));
    cfg.tcp_write_timeout = Some(Duration::from_millis(2000));
    let mut client = tcp::Transport::new_with_cfg("192.168.200.1", cfg)?;
    let mut time;
    loop {
        time = SystemTime::now();
        let res = client.read_holding_registers(32016, 1);
        if let Ok(r) = res {
            println!("Result: {:?}", r);
        }

        // if SystemTime::now().duration_since(time).unwrap() < Duration::from_millis(2000) {
        sleep(Duration::from_millis(3000) - SystemTime::now().duration_since(time).unwrap());
        // }
    }
    Ok(())
}

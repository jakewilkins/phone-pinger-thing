extern crate redis;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate chrono;

use std::thread;
use std::time::Duration;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::fmt;
use std::process::Command;

use chrono::Local;

//use std::fmt::Formatter;
use redis::Commands;

// This is what we're going to decode into. Each field is optional, meaning
// that it doesn't have to be present in TOML.
#[derive(Debug, Deserialize)]
struct Config {
    redis_url: String,
    jake_key: String,
    becca_key: String,
    ping_cmd: String,
    ping_timeout: String,
    poll_period: u64,
    max_misses: u64,
    //global_string: Option<String>,
    //global_integer: Option<u64>,
    //server: Option<ServerConfig>,
    //peers: Option<Vec<PeerConfig>>,
}

impl Config {
    fn ping_timeout_int(&self) -> u64 {
        let tm: u64 = self.ping_timeout.parse().expect("must specify a ping timeout");
        tm
    }
}

impl fmt::Display for Config {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        println!("redis_url: {}", self.redis_url);
        Ok(())
    }
}

fn phone_ip_is_up(ip: String, conf: &Config) -> bool {
    let out = Command::new(conf.ping_cmd.as_str()).arg("-c").arg("1")
        .arg("-W").arg(conf.ping_timeout.as_str()).arg(ip).output().expect("could not ping things.");

    if out.status.success() {
        true
    } else {
        //println!("{:?}", out);
        false
    }
}

fn process_missed_request(key: &String, conn: &redis::Connection, conf: &Config) {
    let missed_key = format!("{}_misses", key);
    let misses = conn.get(&missed_key).unwrap_or(0u64);
    if misses > conf.max_misses {
        let date = Local::now();
        println!("{} max misses reached, cleaning up state because !{}", date.format("[%Y-%m-%d][%H:%M:%S]"), key);

        let _ : () = conn.del(&missed_key).unwrap();
        let _ : () = conn.del(key).unwrap();
    } else {
        //println!("phone wasn't found, bumping missed from: {}", misses);
        let _ : () = conn.incr(&missed_key, 1usize).unwrap();
        let expire = conf.poll_period + conf.ping_timeout_int() + 1;
        let _ : () = conn.expire(&missed_key, expire as usize).unwrap();
    }
}

fn load_config() -> Config {
    let mut cfile = match env::var("CONF") {
        Ok(path) => File::open(path).unwrap(),
        Err(_) => panic!("Must supply a CONF")
    };

    let mut file_str = String::new();

    cfile.read_to_string(&mut file_str).unwrap();

    let decoded: Config = toml::from_str(file_str.as_str()).unwrap();
    decoded
}

fn do_check_for_key(key: &String, con: &redis::Connection, config: &Config) {
    //let mut ip: Option<String>;
    let ip = con.get(key).unwrap();

    match ip {
        Some(ip) => {
            if !phone_ip_is_up(ip, config) {
                process_missed_request(key, con, config)
            } else {
                //println!("phone found!")
            }
        }
        None => ()
    }
}

fn main() {
    let config = load_config();

    let client = redis::Client::open(config.redis_url.as_str()).unwrap();
    let con = client.get_connection().unwrap();

    println!("jakes key is: {}", config.jake_key);
    println!("poll period is: {}", config.poll_period);
    let sleep_duration = Duration::new(config.poll_period, 0);


    loop {
        do_check_for_key(&config.jake_key, &con, &config);
        do_check_for_key(&config.becca_key, &con, &config);

        thread::sleep(sleep_duration);
    }
}

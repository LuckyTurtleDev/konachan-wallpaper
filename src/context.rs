use chrono::{offset::Local, Datelike, Timelike};
use evalexpr::{context_map, EvalexprError, HashMapContext};
use local_ip_address::local_ip;
use std::net::{IpAddr, IpAddr::*, Ipv4Addr};

pub fn get_context() -> Result<HashMapContext, EvalexprError> {
	let time = Local::now();
	let ip = local_ip().unwrap_or_else(|err| {
		eprintln!("error could not get ip addr: {err}");
		IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
	});
	let ip = match ip {
		V4(ip) => ip,
		V6(_) => {
			eprintln!("error could not get ip addr: ipv6 is not supported");
			Ipv4Addr::new(0, 0, 0, 0)
		},
	};

	let contex = context_map! {
	"time.day" => time.day() as i64,
	"time.month" => time.month() as i64,
	"time.hour" => time.hour() as i64,
	"time.minute" => time.minute() as i64,
	"time.ordinal" => time.ordinal() as i64,
	"time.weekday.from_monday" => time.weekday().number_from_monday() as i64,
	"time.weekday.from_sunday" => time.weekday().number_from_sunday() as i64,
	"ipv4" => ip.to_string(),
	"ipv4.0" => ip.octets().get(0).unwrap().clone() as i64,
	"ipv4.1" => ip.octets().get(1).unwrap().clone() as i64,
	"ipv4.2" => ip.octets().get(2).unwrap().clone() as i64,
	"ipv4.3" => ip.octets().get(3).unwrap().clone() as i64,
	}?;
	println!("contex: {:#?}", contex);
	Ok(contex)
}

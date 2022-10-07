use chrono::{offset::Local, Datelike, Timelike};
use evalexpr::{context_map, EvalexprError, HashMapContext};

pub fn get_context() -> Result<HashMapContext, EvalexprError> {
	let time = Local::now();
	context_map! {
	"time.day" => time.day() as i64,
	"time.month" => time.month() as i64,
	"time.hour" => time.hour() as i64,
	"time.minute" => time.minute() as i64,
	"time.ordinal" => time.ordinal() as i64,
	"time.weekday.from_monday" => time.weekday().number_from_monday() as i64,
	"time.weekday.from_sunday" => time.weekday().number_from_sunday() as i64,
	}
}

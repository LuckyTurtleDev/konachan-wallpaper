use evalexpr::context_map;
use neli_wifi::Socket;

pub fn get_context() {
	let socket = Socket::connect();
	let interface = socket.unwrap().get_interfaces_info().unwrap();
	eprintln!("{:?}", interface);
	let context = context_map! {
	"wifi.ssid" => "lucky",
	};
}

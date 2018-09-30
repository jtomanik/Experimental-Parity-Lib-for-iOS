extern crate parity;

use std::ptr;
use std::ffi::CString;
use parity::parity_start_ios;
use parity::parity_rpc_ios;
use std::io;

fn main() {
	// The statements here will be executed when the compiled binary is called

	let mut output = ptr::null_mut();
	let args = CString::new("parity --light --no-ipc").unwrap();

	let mut srcunit = String::new();
	let mut switch = true;

	unsafe {
		let ret1 = parity_start_ios(&mut output, args.as_ptr());
//		let ret2 = parity_rpc_ios(output, query.as_ptr());

		while switch {
			println!("Quit?");
			io::stdin().read_line(&mut srcunit).expect(
				"failed to read src unit",
			);

			if srcunit.trim() == "Q" {
//			println!("doing things right with {}", srcunit);
				switch = false;
			} else {
//			println!("either F or C, not {}", srcunit);
				let query = CString::new("{\"method\":\"eth_syncing\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}").unwrap();
				let ret3 = parity_rpc_ios(output, query.as_ptr());
			}
			srcunit = "".to_string();
		}
	}

	println!("bye bye");
}

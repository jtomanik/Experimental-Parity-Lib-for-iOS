extern crate parity;

use parity::parity_start_ios;
use parity::parity_rpc_ios_query;
use parity::parity_rpc_ios_release;
use std::ptr;
use std::ffi::CString;
use std::ffi::CStr;
use std::io;
use std::alloc::System;
use std::os::raw::c_char;

#[global_allocator]
static A: System = System;

fn main() {

	let mut client = ptr::null_mut();
	let chainspec_path = "/Users/jakubtomanik/github/parity/parity-clib/custom_foundation.json";
	let args_string = format!("parity --light --no-ipc --no-color --chain={}", chainspec_path);
	let args = CString::new(args_string).unwrap();

	let mut srcunit = String::new();
	let mut switch = true;

	unsafe {
		let ret1 = parity_start_ios(&mut client, args.as_ptr());

		while switch {
			println!("Quit?");
			io::stdin().read_line(&mut srcunit).expect(
				"failed to read src unit",
			);

			if srcunit.trim() == "Q" {
				switch = false;
			}
			if srcunit.trim() == "a" {
				let mut response: *mut c_char = ptr::null_mut();
				let query3 = CString::new("{\"method\":\"eth_syncing\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}").unwrap();
				let ret3 = parity_rpc_ios_query(client, query3.as_ptr(), &mut response);
				let response_string = CStr::from_ptr(response).to_string_lossy().into_owned();
				parity_rpc_ios_release(response);
				println!("returned {}", response_string);
			}
			if srcunit.trim() == "b" {
				let mut response: *mut c_char = ptr::null_mut();
				let query3 = CString::new("{\"method\":\"web3_clientVersion\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}").unwrap();
				let ret3 = parity_rpc_ios_query(client, query3.as_ptr(), &mut response);
				let response_string = CStr::from_ptr(response).to_string_lossy().into_owned();
				parity_rpc_ios_release(response);
				println!("returned {}", response_string);
			}
			srcunit = "".to_string();
		}
	}

	println!("bye bye");
}

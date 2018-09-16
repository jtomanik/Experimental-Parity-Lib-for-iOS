// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Note that all the structs and functions here are documented in `parity.h`, to avoid
//! duplicating documentation.

//extern crate libc;

extern crate parity_ethereum;
extern crate panic_hook;

use std::os::raw::{c_char, c_void, c_int};
use std::panic;
use std::ptr;
use std::slice;
use std::str;
use std::ffi::CStr;

#[repr(C)]
pub struct ParityParams {
	pub configuration: *mut c_void,
	pub on_client_restart_cb: Option<extern "C" fn(*mut c_void, *const c_char, usize)>,
	pub on_client_restart_cb_custom: *mut c_void,
}

#[no_mangle]
pub unsafe extern fn parity_config_from_cli(args: *const *const c_char, args_lens: *const usize, len: usize, output: *mut *mut c_void) -> c_int {
	panic::catch_unwind(|| {
		*output = ptr::null_mut();

		let args = {
			let arg_ptrs = slice::from_raw_parts(args, len);
			let arg_lens = slice::from_raw_parts(args_lens, len);

			let mut args = Vec::with_capacity(len + 1);
			args.push("parity".to_owned());

			for (&arg, &len) in arg_ptrs.iter().zip(arg_lens.iter()) {
				let string = slice::from_raw_parts(arg as *const u8, len);
				match String::from_utf8(string.to_owned()) {
					Ok(a) => args.push(a),
					Err(_) => return 1,
				};
			}

			args
		};

		match parity_ethereum::Configuration::parse_cli(&args) {
			Ok(mut cfg) => {
				// Always disable the auto-updater when used as a library.
				cfg.args.arg_auto_update = "none".to_owned();

				let cfg = Box::into_raw(Box::new(cfg));
				*output = cfg as *mut _;
				0
			},
			Err(_) => {
				1
			},
		}
	}).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern fn parity_config_destroy(cfg: *mut c_void) {
	let _ = panic::catch_unwind(|| {
		let _cfg = Box::from_raw(cfg as *mut parity_ethereum::Configuration);
	});
}

#[no_mangle]
pub unsafe extern fn parity_start_default(output: *mut *mut c_void, args: *const c_char) -> c_int {
	panic::catch_unwind(|| {
		*output = ptr::null_mut();
		let argument_string = CStr::from_ptr(args).to_string_lossy().into_owned();
		let arguments: Vec<&str> = argument_string.split(' ').collect();

		println!("Arguments received: {}", argument_string);
		println!("Arguments parsed: {}", arguments);

		let config = {
			parity_ethereum::Configuration::parse_cli(&arguments).unwrap_or_else(|e| e.exit())
		};
//		print_config(config);

		let directories = config.directories();

		println!("IPC config state: {}", config.args.flag_no_ipc);
		println!("Base path args:{:?}, dirs:{}", config.args.arg_base_path, directories.base);
		println!("DB path args:{:?}, dirs:{} ", config.args.arg_db_path, directories.db);
		println!("Cache dir:{} ", directories.cache);
		println!("Keys dir:{} ", directories.keys);
		println!("Secretstore dir:{} ", directories.secretstore);
		println!("No warp state: {}", config.args.flag_no_warp);
		println!("Light client state: {}", config.args.flag_light);
		println!("Secretstore state: {}", config.args.flag_no_secretstore);
		println!("Whisper state: {}", config.args.flag_whisper);

		let on_client_restart_cb = |_ : String| {};
		let action = match parity_ethereum::start(config, on_client_restart_cb, || {}) {
			Ok(action) => action,
			Err(msg) => {
				println!("{}", msg);
				return 1
			}
		};

		match action {
			parity_ethereum::ExecutionAction::Instant(Some(s)) => { println!("{}", s); 0 },
			parity_ethereum::ExecutionAction::Instant(None) => 0,
			parity_ethereum::ExecutionAction::Running(client) => {
				println!("address of the running client {:p}", &client);
				let pointer = Box::into_raw(Box::<parity_ethereum::RunningClient>::new(client)) as *mut c_void;
				println!("address of the boxed client {:p}", pointer);
				println!("address of the box {:p}", &pointer);
				*output = pointer;
				0
			}
		}
	}).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern fn parity_start(cfg: *const ParityParams, output: *mut *mut c_void) -> c_int {
	panic::catch_unwind(|| {
		*output = ptr::null_mut();
		let cfg: &ParityParams = &*cfg;

		let config = Box::from_raw(cfg.configuration as *mut parity_ethereum::Configuration);

		let on_client_restart_cb = {
			let cb = CallbackStr(cfg.on_client_restart_cb, cfg.on_client_restart_cb_custom);
			move |new_chain: String| { cb.call(&new_chain); }
		};

		let action = match parity_ethereum::start(*config, on_client_restart_cb, || {}) {
			Ok(action) => action,
			Err(_) => return 1,
		};

		match action {
			parity_ethereum::ExecutionAction::Instant(Some(s)) => { println!("{}", s); 0 },
			parity_ethereum::ExecutionAction::Instant(None) => 0,
			parity_ethereum::ExecutionAction::Running(client) => {
				*output = Box::into_raw(Box::<parity_ethereum::RunningClient>::new(client)) as *mut c_void;
				0
			}
		}
	}).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern fn parity_destroy(client: *mut c_void) {
	let _ = panic::catch_unwind(|| {
		let client = Box::from_raw(client as *mut parity_ethereum::RunningClient);
		client.shutdown();
	});
}

#[no_mangle]
pub unsafe extern fn parity_rpc(client: *mut c_void, query: *const char, len: usize, out_str: *mut c_char, out_len: *mut usize) -> c_int {
	panic::catch_unwind(|| {
		let client: &mut parity_ethereum::RunningClient = &mut *(client as *mut parity_ethereum::RunningClient);

		let query_str = {
			let string = slice::from_raw_parts(query as *const u8, len);
			match str::from_utf8(string) {
				Ok(a) => a,
				Err(_) => return 1,
			}
		};

		if let Some(output) = client.rpc_query_sync(query_str) {
			let q_out_len = output.as_bytes().len();
			if *out_len < q_out_len {
				return 1;
			}

			ptr::copy_nonoverlapping(output.as_bytes().as_ptr(), out_str as *mut u8, q_out_len);
			*out_len = q_out_len;
			0
		} else {
			1
		}
	}).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern fn parity_set_panic_hook(callback: extern "C" fn(*mut c_void, *const c_char, usize), param: *mut c_void) {
	let cb = CallbackStr(Some(callback), param);
	panic_hook::set_with(move |panic_msg| {
		cb.call(panic_msg);
	});
}

// Internal structure for handling callbacks that get passed a string.
struct CallbackStr(Option<extern "C" fn(*mut c_void, *const c_char, usize)>, *mut c_void);
unsafe impl Send for CallbackStr {}
unsafe impl Sync for CallbackStr {}
impl CallbackStr {
	fn call(&self, new_chain: &str) {
		if let Some(ref cb) = self.0 {
			cb(self.1, new_chain.as_bytes().as_ptr() as *const _, new_chain.len())
		}
	}
}

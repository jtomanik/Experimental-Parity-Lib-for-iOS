extern crate parity;

use std::ptr;
use std::ffi::CString;
use parity::parity_start_ios;

fn main() {
	// The statements here will be executed when the compiled binary is called

	let mut output = ptr::null_mut();

	unsafe {
		let args = CString::new("parity --light --no-ipc").unwrap();
		let ret = parity_start_ios(&mut output, args.as_ptr());
		println!("returned {}", ret);
		println!("output {:p}", output);
		println!("& mut output {:p}", &mut output);
	}
}

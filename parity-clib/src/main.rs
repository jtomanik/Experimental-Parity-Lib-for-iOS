extern crate parity;

use std::ptr;
use parity::parity_start_default;

fn main() {
	// The statements here will be executed when the compiled binary is called

	let mut output = ptr::null_mut();

	unsafe {
		let ret = parity_start_default(&mut output);
		println!("returned {}", ret);
		println!("output {:p}", output);
		println!("& mut output {:p}", &mut output);
	}
}

use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
	let args: Vec<String> = std::env::args().collect();

	let file = if args.len() > 1 {
		&args[1]
	} else {
		"challenge.bin"
	};

	let mut data = Vec::new();
	File::open(Path::new(file))
		.expect(&format!("Couldn't open file {}", file))
		.read_to_end(&mut data)
		.expect(&format!("Couldn't read file {}", file));

	let mut memory: Vec<u16> = Vec::new();

	for i in (0..data.len()).step_by(2) {
		// stored as little endian: low high
		memory.push((data[i + 1] as u16) << 8 | (data[i] as u16));
	}

	let _registers: [u16; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
	let _stack: Vec<u16> = Vec::new();
	let mut pc = 0;

	let mut next = || {
		let code = memory[pc];
		pc += 1;
		return code;
	};

	loop {
		match next() {
			19 => print!("{}", next() as u8 as char),
			21 => (),
			v => panic!(format!("unknown opcode {}", v)),
		};
	}
}

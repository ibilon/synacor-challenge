use std::fs::File;
use std::io::Read;
use std::path::Path;

enum Value {
	Literal(u16),
	Register(u16),
}

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

	//*
	let mut memory: Vec<u16> = Vec::new();

	for i in (0..data.len()).step_by(2) {
		// stored as little endian: low high
		memory.push((data[i + 1] as u16) << 8 | (data[i] as u16));
	}
	// */
	// let memory = vec![9, 32768, 32758, 15 + '0' as u16, 19, 32768, 0];

	let mut registers: [u16; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
	let mut stack: Vec<u16> = Vec::new();
	let mut pc = 0;

	fn parse(word: u16) -> Value {
		return if word <= 32767 {
			Value::Literal(word)
		} else if word <= 32775 {
			Value::Register(word - 32768)
		} else {
			panic!(format!("Value {} is invalid", word));
		};
	}

	fn get(address: usize, registers: &[u16; 8], memory: &Vec<u16>) -> u16 {
		return match parse(memory[address]) {
			Value::Literal(v) => v,
			Value::Register(r) => registers[r as usize],
		};
	}

	fn set(address: usize, value: u16, registers: &mut [u16; 8], memory: &mut Vec<u16>) {
		match parse(memory[address]) {
			// Value::Literal(v) => panic!(format!("set address is literal {}", v)),
			Value::Literal(v) => memory[v as usize] = value,
			Value::Register(r) => registers[r as usize] = value,
		}
	}

	loop {
		match memory[pc] {
			// halt
			0 => break,

			// set a b
			1 => {
				let b = get(pc + 2, &registers, &memory);

				set(pc + 1, b, &mut registers, &mut memory);

				pc += 3;
			}

			// push a
			2 => {
				stack.push(get(pc + 1, &registers, &memory));
				pc += 2;
			}

			// pop a
			3 => {
				match stack.pop() {
					None => panic!("pop but stack is empty"),
					Some(v) => set(pc + 1, v, &mut registers, &mut memory),
				};
				pc += 2;
			}

			// eq a b c
			4 => {
				let b = get(pc + 2, &registers, &memory);
				let c = get(pc + 3, &registers, &memory);

				set(
					pc + 1,
					if b == c { 1 } else { 0 },
					&mut registers,
					&mut memory,
				);

				pc += 4;
			}

			// gt a b c
			5 => {
				let b = get(pc + 2, &registers, &memory);
				let c = get(pc + 3, &registers, &memory);

				set(
					pc + 1,
					if b > c { 1 } else { 0 },
					&mut registers,
					&mut memory,
				);

				pc += 4;
			}

			// jmp a
			6 => pc = get(pc + 1, &registers, &memory) as usize,

			// jt a b
			7 => {
				let a = get(pc + 1, &registers, &memory);
				let b = get(pc + 2, &registers, &memory);

				if a != 0 {
					pc = b as usize;
				} else {
					pc += 3;
				}
			}

			// jf a b
			8 => {
				let a = get(pc + 1, &registers, &memory);
				let b = get(pc + 2, &registers, &memory);

				if a == 0 {
					pc = b as usize;
				} else {
					pc += 3;
				}
			}

			// add a b c
			9 => {
				let b = get(pc + 2, &registers, &memory) as u32;
				let c = get(pc + 3, &registers, &memory) as u32;

				set(
					pc + 1,
					((b + c) % 32768) as u16,
					&mut registers,
					&mut memory,
				);

				pc += 4;
			}

			// mult a b c
			10 => {
				let b = get(pc + 2, &registers, &memory) as u32;
				let c = get(pc + 3, &registers, &memory) as u32;

				set(
					pc + 1,
					((b * c) % 32768) as u16,
					&mut registers,
					&mut memory,
				);

				pc += 4;
			}

			// mod a b c
			11 => {
				let b = get(pc + 2, &registers, &memory);
				let c = get(pc + 3, &registers, &memory);

				set(pc + 1, b % c, &mut registers, &mut memory);

				pc += 4;
			}

			// and a b c
			12 => {
				let b = get(pc + 2, &registers, &memory);
				let c = get(pc + 3, &registers, &memory);

				set(pc + 1, b & c, &mut registers, &mut memory);

				pc += 4;
			}

			// or a b c
			13 => {
				let b = get(pc + 2, &registers, &memory);
				let c = get(pc + 3, &registers, &memory);

				set(pc + 1, b | c, &mut registers, &mut memory);

				pc += 4;
			}

			// not a b
			14 => {
				let b = get(pc + 2, &registers, &memory);

				set(pc + 1, (!b) & 32767, &mut registers, &mut memory);

				pc += 3;
			}

			// rmem a b
			15 => {
				let b = get(
					get(pc + 2, &registers, &memory) as usize,
					&registers,
					&memory,
				);

				set(pc + 1, b, &mut registers, &mut memory);

				pc += 3;
			}

			// wmem a b
			16 => {
				let a = get(pc + 1, &registers, &memory);
				let b = get(pc + 2, &registers, &memory);

				memory[a as usize] = b;

				pc += 3;
			}

			// call a
			17 => {
				stack.push((pc + 2) as u16);
				pc = get(pc + 1, &registers, &memory) as usize;
			}

			// ret
			18 => match stack.pop() {
				None => break,
				Some(val) => pc = val as usize,
			},

			// out a
			19 => {
				print!("{}", get(pc + 1, &registers, &memory) as u8 as char);
				pc += 2;
			}

			// in a
			20 => {
				panic!("op 20 not implemented");
			}

			// noop
			21 => pc += 1,

			v => panic!(format!("unknown opcode {}", v)),
		};
	}
}

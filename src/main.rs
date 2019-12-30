use std::fs::File;
use std::io::Read;
use std::path::Path;

enum Value {
	Literal(u16),
	Register(u16),
	Invalid(u16),
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

	let mut memory: Vec<u16> = Vec::new();

	for i in (0..data.len()).step_by(2) {
		// stored as little endian: low high
		memory.push((data[i + 1] as u16) << 8 | (data[i] as u16));
	}

	let mut registers: [u16; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
	let mut stack: Vec<u16> = Vec::new();
	let mut pc = 0;

	fn parse(word: u16) -> Value {
		if word <= 32767 {
			return Value::Literal(word);
		} else if word <= 32775 {
			return Value::Register(word - 32768);
		} else {
			return Value::Invalid(word);
		}
	}

	fn get(value: Value, registers: [u16; 8]) -> u16 {
		return match value {
			Value::Literal(v) => v,
			Value::Register(r) => registers[r as usize],
			Value::Invalid(v) => panic!(format!("Using {} as value", v)),
		};
	}

	fn set(a: Value, b: u16, mut registers: [u16; 8]) {
		match a {
			Value::Literal(_) => panic!("set address is literal"),
			Value::Register(r) => registers[r as usize] = b,
			Value::Invalid(_) => panic!("set address is invalid"),
		}
	}

	loop {
		match memory[pc] {
			// halt
			0 => break,

			// set a b
			1 => {
				match parse(memory[pc + 1]) {
					Value::Register(r) => {
						registers[r as usize] = get(parse(memory[pc + 2]), registers)
					}

					_ => panic!("set on non register"),
				}

				pc += 3;
			}

			// push a
			2 => {
				stack.push(get(parse(memory[pc + 1]), registers));
				pc += 2;
			}

			// pop a
			3 => {
				match stack.pop() {
					None => panic!("pop but stack is empty"),
					Some(v) => set(parse(memory[pc + 1]), v, registers),
				};
				pc += 2;
			}

			// jmp a
			6 => pc = get(parse(memory[pc + 1]), registers) as usize,

			// jt a b
			7 => {
				let a = get(parse(memory[pc + 1]), registers);
				let b = get(parse(memory[pc + 2]), registers);

				if a != 0 {
					pc = b as usize;
				} else {
					pc += 3;
				}
			}

			// jf a b
			8 => {
				let a = get(parse(memory[pc + 1]), registers);
				let b = get(parse(memory[pc + 2]), registers);

				if a == 0 {
					pc = b as usize;
				} else {
					pc += 3;
				}
			}

			// call a
			17 => {
				stack.push(pc as u16);
				pc = get(parse(memory[pc + 1]), registers) as usize;
			}

			// ret
			18 => match stack.pop() {
				None => break,
				Some(val) => pc = val as usize,
			},

			// out a
			19 => {
				print!("{}", memory[pc + 1] as u8 as char);
				pc += 2;
			}

			// noop
			21 => pc += 1,

			v => panic!(format!("unknown opcode {}", v)),
		};
	}
}

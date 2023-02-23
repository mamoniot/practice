
fn to_num(n: u64, default_radix: u32, max_radix: u32, ret_str: &mut [u32]) {
	let len = ret_str.len() as u32;
	if len < 1 {return;}
	let first_place = (default_radix as u64).pow(len - 1);
	let mut first = (n/first_place) as u32;
	let mut m;
	if first > default_radix {
		first = default_radix;
		m = first_place*first as u64;
		loop {
			let next_m = m + (first as u64 + 1).pow(len - 1);
			if n < next_m {break;}
			first += 1;
			m = next_m;
		}
	} else {
		m = first_place*first as u64;
	}
	let radix = (first as u64 + 1).max(default_radix as u64);
	if radix as u32 > max_radix {
		ret_str.fill(max_radix - 1);
		return;
	}
	let mut r = n - m;
	for v in (&mut ret_str[1..]).iter_mut().rev() {
		*v = (r%radix) as u32;
		r /= radix;
	}
	ret_str[0] = first;
}

fn from_num(str: &[u32], default_radix: u32) -> u64 {
	let len = str.len() as u32;
	let first = match str.first() {
		Some(n) => *n as u64,
		None => return 0,
	};
	let mut m = (default_radix as u64).pow(len - 1)*(default_radix as u64).min(first);
	for i in (default_radix as u64 + 1)..(first + 1) {
		m += i.pow(len - 1);
	}
	let mut n = 0;
	let radix = (first + 1).max(default_radix as u64);
	for v in &str[1..] {
		n *= radix;
		n += *v as u64;
	}
	return m + n;
}



fn main() {
	let mut s = [0; 3];

	let radix = 10;
	for i in 0..100000 {
		let n = i + 5000;
		to_num(n, radix, 36, &mut s);
		let m = from_num(&s, radix);
		let mut c = [0u8; 3];
		for (i, v) in s.iter().enumerate() {
			if *v < 10 {
				c[i] = *v as u8 + 48;
			} else {
				c[i] = *v as u8 + 87;
			}
		}
		let b = std::str::from_utf8(&c);
		println!("{:?}:{:?}:{:?}", n, m, b.unwrap());
		//assert_eq!(m, n);
	}
}

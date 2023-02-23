mod pool;

use pool::*;




fn main() {
	let mut pool = Pool::new();
	let mut v = Vec::new();

	let b = pool.alloc(561435);

	for i in 0..100 {
		let a = pool.alloc(i);
		print!("{}, ", *a);
		v.push(a);
	}
	let c = &*b;

	for i in 0..100 {
		let a = pool.alloc(i);
		print!("{}, ", *a);
		v.push(a);
	}

	print!("{}, ", *c);
}

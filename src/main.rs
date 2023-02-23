mod pool;

use pool::*;




fn main() {
	let pool: Pool<i32> = Pool::new();
	let mut v = Vec::new();

	let b = pool.alloc_ref(561435);

	for i in 0..100 {
		let a = pool.alloc_ref(i);
		print!("{}, ", *a);
		//v.push(a);
	}
	let c = &*b;

	for i in 0..100 {
		let a = pool.alloc_raii(i);
		print!("{}, ", *a);
		v.push(a);
	}

	print!("{}, ", *c);
}

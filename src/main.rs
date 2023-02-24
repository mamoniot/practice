mod pool;

use pool::*;

struct test (i64, i64);
impl Drop for test {
	fn drop(&mut self) {
		println!("drop {}", self.0);
	}
}



fn main() {
	let pool = Pool::new();
	let mut v = Vec::new();

	let b = pool.alloc_ref(test(561435, -1));

	for i in 0..100 {
		let a = pool.alloc_ref(test(i, i));
		print!("{}, ", a.0);
		v.push(a);
	}

	//for i in 0..100 {
		//let a = pool.alloc_raii(test(i));
		//print!("{}, ", a.0);
		//v.push(a);
	//}

	//pool.free_ref(b);

	print!("{}, ", b.0);
}

mod pool;

use pool::*;

struct Test (usize, usize);
impl Drop for Test {
	fn drop(&mut self) {
		println!("drop {}", self.0);
	}
}

fn main() {
	const N: usize = 10;
	let pool = Pool::<_, 100>::with_capacity();
	let mut v: [_; N] = std::array::from_fn(|i| pool.alloc_raii(Test(i, i)));

	for j in 0..N {
		let i = (j*13)%N;
		let p = pool.alloc_raii(Test(v[i].0 + N, i));
		println!("add  {}", p.0);
		v[i] = p;
	}
}

use Pool;

fn test() {
	let pool = Pool::new();

	let a = pool.alloc();

	println!("{}", a);
}

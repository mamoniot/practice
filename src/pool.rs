use std::{sync::Mutex, mem::{MaybeUninit, size_of}, ptr, cell::UnsafeCell, ops::{DerefMut, Deref}};

const DEFAULT_L: usize = 64;

struct PoolMem<T, const L: usize = DEFAULT_L> {
	mem: [MaybeUninit<T>; L],
	pre: *mut PoolMem<T, L>,
	size: usize,
}

pub struct Pool<T, const L: usize = DEFAULT_L> {
	first_free: Mutex<*mut T>,
	head_arena: UnsafeCell<*mut PoolMem<T, L>>,
}

pub struct PoolRef<'a, 'b, T, const L: usize = DEFAULT_L> {
	pub ptr: &'b mut T,
	pub origin_pool: &'a Pool<T, L>,
}


impl<T> Pool<T> {
	pub fn new() -> Self {
		Self::with_capacity()
	}
}


impl<T, const L: usize> Pool<T, L> {
	pub fn with_capacity() -> Self {
		debug_assert!(size_of::<T>() >= size_of::<*mut T>());
		Pool {
			first_free: Mutex::new(std::ptr::null_mut()),
			head_arena: UnsafeCell::new(Box::leak(Box::new(PoolMem {
				pre: ptr::null_mut(),
				size: 0,
				mem: std::array::from_fn(|_| MaybeUninit::uninit()),
			}))),
		}
	}

	pub fn alloc_ptr(&self) -> *mut T {
		unsafe {
			let mut mutex = self.first_free.lock().unwrap();
			if mutex.is_null() {
				let mut arena = *self.head_arena.get();
				if (*arena).size >= L {
					let new = Box::leak(Box::new(PoolMem {
						pre: arena,
						size: 0,
						mem: std::array::from_fn(|_| MaybeUninit::uninit()),
					}));
					*self.head_arena.get() = new;
					arena = new;
				}
				let slot = &mut (*arena).mem[(*arena).size];
				(*arena).size += 1;
				slot.as_mut_ptr()
			} else {
				let ptr = *mutex;
				*mutex = *ptr.cast::<*mut T>();
				ptr
			}
		}
	}
	pub fn free_ptr(&self, ptr: *mut T) {
		unsafe {
			//linked-list insert
			let mut mutex = self.first_free.lock().unwrap();
			*ptr.cast::<*mut T>() = *mutex;
			*mutex = ptr;
		}
	}

	pub fn alloc_ref(&self, obj: T) -> &mut T {
		unsafe {
			let ptr = self.alloc_ptr();
			ptr.write(obj);
			&mut *ptr
		}
	}
	pub fn free_ref(&self, ptr: &mut T) {
		let ptr: *mut T = ptr;
		unsafe {
			ptr.drop_in_place();
			self.free_ptr(ptr);
		}
	}

	pub fn alloc_raii(&self, obj: T) -> PoolRef<T, L> {
		PoolRef {
			ptr: self.alloc_ref(obj),
			origin_pool: self,
		}
	}
}


impl<T, const L: usize> Drop for Pool<T, L> {
	fn drop(&mut self) {
		let mutex = self.first_free.lock().unwrap();
		unsafe {
			let mut arena = *self.head_arena.get();
			while !arena.is_null() {
				let mem = Box::from_raw(arena);
				arena = mem.pre;
				drop(mem);
			}
		}
		drop(mutex);
	}
}


impl<'a, 'b, T, const L: usize> Deref for PoolRef<'a, 'b, T, L> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.ptr
	}
}
impl<'a, 'b, T, const L: usize> DerefMut for PoolRef<'a, 'b, T, L> {
	fn deref_mut(&mut self) -> &mut T {
		self.ptr
	}
}
impl<'a, 'b, T, const L: usize> Drop for PoolRef<'a, 'b, T, L> {
	fn drop(&mut self) {
		self.origin_pool.free_ref(self.ptr);
	}
}

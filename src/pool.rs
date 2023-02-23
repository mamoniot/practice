use std::{sync::Mutex, mem::{MaybeUninit, size_of}, ptr, cell::UnsafeCell, ops::{DerefMut, Deref}};

const PAGE_SIZE: usize = 1<<12;
const L: usize = PAGE_SIZE - 2*size_of::<usize>();

struct PoolMem<T> {
	mem: [MaybeUninit<u8>; L],
	pre: *mut PoolMem<T>,
	size: usize,
}

#[derive(Debug)]
pub struct Pool<T> {
	first_free: Mutex<*mut T>,
	head_arena: UnsafeCell<*mut PoolMem<T>>,
}

#[derive(Debug)]
pub struct PoolRef<'a, 'b, T> {
	ptr: &'b mut T,
	origin_pool: &'a Pool<T>,
}


impl<T> Pool<T> {
	pub fn new() -> Self {
		debug_assert!(size_of::<T>() >= size_of::<*mut T>());
		debug_assert!(size_of::<T>() < L);
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
				let slot = &mut (*arena).mem[(*arena).size * size_of::<T>()];
				(*arena).size += 1;

				slot.as_mut_ptr().cast()
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

	pub fn alloc_raii(&self, obj: T) -> PoolRef<T> {
		PoolRef {
			ptr: self.alloc_ref(obj),
			origin_pool: self,
		}
	}
}


impl<T> Drop for Pool<T> {
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


impl<'a, 'b, T> Deref for PoolRef<'a, 'b, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.ptr
	}
}
impl<'a, 'b, T> DerefMut for PoolRef<'a, 'b, T> {
	fn deref_mut(&mut self) -> &mut T {
		self.ptr
	}
}
impl<'a, 'b, T> Drop for PoolRef<'a, 'b, T> {
	fn drop(&mut self) {
		self.origin_pool.free_ref(self.ptr);
	}
}

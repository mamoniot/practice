use std::{sync::Mutex, cell::UnsafeCell, ops::{Deref, DerefMut}, mem::{MaybeUninit, ManuallyDrop}};

union PoolSlot<T> {
	obj: ManuallyDrop<MaybeUninit<T>>,
	next: usize,
}

pub struct Pool<T> {
	arena: UnsafeCell<Vec<PoolSlot<T>>>,
	first_free: Mutex<usize>,
}
pub struct PoolRef<'a, T> {
	idx: usize,
	origin_pool: &'a Pool<T>,
}

impl<T> Pool<T> {
	pub fn new() -> Self {
		Pool {
			first_free: Mutex::new(usize::MAX),
			arena: UnsafeCell::new(Vec::new()),
		}
	}
	pub fn alloc<'a>(&'a self, obj: T) -> PoolRef<'a, T> {
		let arena = unsafe { &mut *self.arena.get() };

		let mut mutex = self.first_free.lock().unwrap();
		let idx = if *mutex == usize::MAX {
			arena.push(PoolSlot {obj: ManuallyDrop::new(MaybeUninit::new(obj))});
			arena.len() - 1
		} else {
			let idx = *mutex;
			unsafe {
				*mutex = arena[idx].next;
				arena[idx].obj.write(obj);
			}
			idx
		};
		PoolRef {idx, origin_pool: self}
	}
	fn free(&self, idx: usize) {
		unsafe {
			//drop
			let arena = &mut *self.arena.get();
			arena[idx].obj.assume_init_drop();
			//linked-list insert
			let mut mutex = self.first_free.lock().unwrap();
			arena[idx].next = *mutex;
			*mutex = idx;
		}
	}
}

impl<T> Drop for Pool<T> {
	fn drop(&mut self) {
		let mutex = self.first_free.lock().unwrap();
		unsafe {
			//drop
			let arena = &mut *self.arena.get();
			alloca::with_alloca_zeroed((arena.len() + 7)/8, |buffer| {
				let mut free = *mutex;
				while free != usize::MAX {
					let idx = free/8;
					let bit = free%8;
					buffer[idx] |= 1u8<<bit;//mark the index as unoccupied
					free = arena[free].next;
				}

				for i in 0..arena.len() {
					let idx = i/8;
					let bit = i%8;
					if (buffer[idx]>>bit)&1 == 0 {
						arena[i].obj.assume_init_drop();
					}
				}
			});
		}
	}
}

impl<'a, T> Deref for PoolRef<'a, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		unsafe {
			(&*self.origin_pool.arena.get())[self.idx].obj.assume_init_ref()
		}
	}
}
impl<'a, T> DerefMut for PoolRef<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			(&mut *self.origin_pool.arena.get())[self.idx].obj.assume_init_mut()
		}
	}
}
impl<'a, T> Drop for PoolRef<'a, T> {
	fn drop(&mut self) {
		self.origin_pool.free(self.idx);
	}
}

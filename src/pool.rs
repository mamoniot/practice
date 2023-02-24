use std::{sync::Mutex, mem::{MaybeUninit, size_of}, ptr, cell::UnsafeCell, ops::{DerefMut, Deref}};

const DEFAULT_L: usize = 64;


struct PoolMem<T, const L: usize = DEFAULT_L> {
	mem: [MaybeUninit<T>; L],
	pre: *mut PoolMem<T, L>,
	size: usize,
}

/// A generic, thread-safe, fixed-sized memory allocator for instances of `T`.
/// New instances of `T` are packed together into arrays of size `L`, and allocated in bulk as one large page from the global allocator.
/// Pages from the global allocator are not deallocated until the pool is dropped, and are re-used as instances of `T` are allocated and freed.
///
/// Objects can be allocated as raw pointers, in which case they must be manually initialized, dropped and freed, or they can be allocated as a rust-style RAII wrapper that initializes, drops and frees memory automatically.
///
/// Allocating from a pool results in almost no internal and external fragmentation in the global heap, thus saving significant amounts of memory from being used by one's program. Pools also allocate memory significantly faster on average than the global allocator, you can expect a `Pool<T>` allocation to outperform `Box<T>`. It can also have better cpu caching characteristics increasing runtime performance.
pub struct Pool<T, const L: usize = DEFAULT_L> {
	first_free: Mutex<*mut T>,
	head_arena: UnsafeCell<*mut PoolMem<T, L>>,
}

/// A rust-style RAII wrapper that drops and frees memory allocated from a pool automatically, the same as a `Box<T>`. This will run the destructor of `T` in place within the pool before freeing it.
pub struct PoolBox<'a, 'b, T, const L: usize = DEFAULT_L> {
	pub ptr: &'b mut T,
	pub origin_pool: &'a Pool<T, L>,
}


impl<T> Pool<T> {
	/// Creates a new `Pool<T>` with the default packing length.
	pub fn new() -> Self {
		Self::with_capacity()
	}
}


impl<T, const L: usize> Pool<T, L> {
	/// Creates a new `Pool<T>` with packing length `L`. Packing length determines the number of instances of `T` that will fit in a page before it becomes full. Once all pages in a `Pool<T>` are full a new page is allocated from the global allocator. Larger values of `L` are generally faster, but the returns are diminishing and vary by platform.
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

	/// Allocates uninitialized memory for an instance `T`. The returned pointer points to this memory. It is undefined what will be contained in this memory, it must be initiallized before being used. This pointer must be manually freed from the pool using `Pool::free_ptr` before being dropped, otherwise its memory will be leaked. In addition, if `T` has a destructor, then `ptr::mut_ptr::drop_in_place()` must be called on the pointer before it is freed, as pointers do not trigger destructors on their own. If the pool is dropped before this pointer is freed, the destructor of `T` will not be run and this pointer will point to invalid memory.
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
	/// Frees memory allocated from the pool by `Pool::alloc_ptr`. This must be called only once on only pointers returned by `Pool::alloc_ptr` from the same pool. Once memory is freed the content of the memory is undefined, it should not be read or written.
	///
	/// If `T` has a destructor, then `ptr::mut_ptr::drop_in_place()` must be called on the pointer before it is freed, as pointers do not trigger destructors on their own.
	pub fn free_ptr(&self, ptr: *mut T) {
		unsafe {
			//linked-list insert
			let mut mutex = self.first_free.lock().unwrap();
			*ptr.cast::<*mut T>() = *mutex;
			*mutex = ptr;
		}
	}


	/// Allocates memory for an instance `T` initialized with the contents of `obj`. The returned reference is for this object. This reference must be manually freed from the pool using `Pool::free_ref` before being dropped, otherwise its memory will be leaked.
	pub fn alloc_ref(&self, obj: T) -> &mut T {
		unsafe {
			let ptr = self.alloc_ptr();
			ptr.write(obj);
			&mut *ptr
		}
	}
	/// Frees an object allocated from a pool by `Pool::alloc_ref`. The object will be dropped, calling the destructor of `T` in place within the pool. This must be called only once on only references returned by `Pool::alloc_ref` from the same pool. Once memory is freed the content of the memory is undefined, it should not be read or written. The reference ought to be immediately dropped after `free_ref` is called.
	pub fn free_ref(&self, ptr: &mut T) {
		let ptr: *mut T = ptr;
		unsafe {
			ptr.drop_in_place();
			self.free_ptr(ptr);
		}
	}

	/// Allocates memory for an instance `T` initialized with the contents of `obj`. The returned `PoolBox<T>` is for this object. A `PoolBox<T>` will automatically drop the object and free itself when it is dropped, making memory leaks, use-after-free and skipped destructors impossible. This method is similar in function to `Box::new()`.
	pub fn alloc(&self, obj: T) -> PoolBox<T, L> {
		PoolBox {
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


impl<'a, 'b, T, const L: usize> Deref for PoolBox<'a, 'b, T, L> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.ptr
	}
}
impl<'a, 'b, T, const L: usize> DerefMut for PoolBox<'a, 'b, T, L> {
	fn deref_mut(&mut self) -> &mut T {
		self.ptr
	}
}
impl<'a, 'b, T, const L: usize> Drop for PoolBox<'a, 'b, T, L> {
	fn drop(&mut self) {
		self.origin_pool.free_ref(self.ptr);
	}
}

/*=====================================================================*\
** NotVeryMoe Astro | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*=====================================================================*/

#[derive(Debug)]
pub struct RingBuffer<T> {
    data:     *mut T,
    head:     u16,
    tail:     u16,
    length:   u16,
    capacity: u16,
}

unsafe impl<T: Sync> Sync for RingBuffer<T> {}
unsafe impl<T: Send> Send for RingBuffer<T> {}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        unsafe { core::mem::drop(Box::from_raw(core::slice::from_raw_parts_mut(self.data, self.capacity as usize))) }
    }
}

impl<T: Default + Copy> RingBuffer<T> {
    pub fn new(capacity: u16) -> Self {
        Self{
            data: Box::leak(vec![T::default(); capacity as usize].into_boxed_slice()).as_mut_ptr(),
            head: 0,
            tail: 0,
            length: 0,
            capacity,
        }
    }
}

impl<T: Copy> RingBuffer<T> {

    pub fn push_front(&mut self, value: T) {
        debug_assert!(self.length < self.capacity);
        self.head = wrap_dec(self.head, self.capacity);
        unsafe{ *self.data.offset(self.head as isize) = value; } 
        self.length += 1;
    }

    pub fn push_back(&mut self, value: T) {
        debug_assert!(self.length < self.capacity);
        unsafe{ *self.data.offset(self.tail as isize) = value; } 
        self.tail = wrap_inc(self.tail, self.capacity);
        self.length += 1;
    }

    pub fn pop_front(&mut self) -> T {
        debug_assert!(self.length > 0);
        let old_head = self.head  as isize;
        self.head = wrap_inc(self.head, self.capacity);
        self.length -= 1;
        unsafe { *self.data.offset(old_head) }
    }

    pub fn pop_back(&mut self) -> T {
        debug_assert!(self.length > 0);
        self.tail = wrap_dec(self.tail, self.capacity);
        self.length -= 1;
        unsafe { *self.data.offset(self.tail as isize) }
    }

    pub fn front(&self) -> &T {
        unsafe { &*self.data.offset(self.head as isize) }
    }

    pub fn front_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.offset(self.head as isize) }
    }

    pub fn back(&self) -> &T {
        unsafe { &*self.data.offset(wrap_dec(self.tail, self.capacity) as isize) }
    }

    pub fn back_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.offset(wrap_dec(self.tail, self.capacity) as isize) }
    }

    pub fn len(&self) -> u16 {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn is_full(&self) -> bool {
        self.length >= self.capacity
    }

    pub fn capacity(&self) -> u16 {
        self.capacity
    }

    pub fn get(&self, index: u16) -> &T {
        debug_assert!(index < self.length);
        let index = wrap_fast(self.head, index, self.capacity);
        unsafe { &mut *self.data.offset(index as isize) }
    }

    pub fn get_mut(&mut self, index: u16) -> &mut T {
        debug_assert!(index < self.length);
        let index = wrap_fast(self.head, index, self.capacity);
        unsafe { &mut *self.data.offset(index as isize) }
    }
}

#[inline] fn wrap_fast(value: u16, distance: u16, max: u16) -> isize {
    let max_dist = max - value;
    (if max_dist < distance {
        distance - max_dist
    } else {
        value + distance
    }) as isize
}

#[inline] fn wrap_inc(value: u16, max: u16) -> u16 {
    (value + 1) % max
    /*if value + 1 >= max {
        0
    } else {
        value + 1
    }*/
}

#[inline] fn wrap_dec(value: u16, max: u16) -> u16 {
    if value == 0 {
        max - 1
    } else {
        value - 1
    }
}
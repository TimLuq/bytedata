use alloc::sync::Arc;
use core::sync::atomic::AtomicPtr;

use crate::ByteData;

/// A static interning structure that never releases the interned values.
pub struct StaticInterning {
    max_len: usize,
    map: AtomicPtr<dashmap::DashMap<ByteData<'static>, ()>>,
}

impl StaticInterning {
    /// Creates a new uninitialized static interning instance.
    #[inline]
    #[must_use]
    pub const fn new_const(max_len: usize) -> Self {
        Self {
            max_len,
            map: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Creates a new static interning instance.
    #[inline]
    #[must_use]
    pub fn new(max_len: usize) -> Self {
        Self {
            max_len,
            map: AtomicPtr::new(Arc::into_raw(Arc::new(
                dashmap::DashMap::<ByteData<'static>, ()>::new(),
            ))
                as *mut dashmap::DashMap<ByteData<'static>, ()>),
        }
    }

    /// Clears the interning set.
    #[inline]
    pub fn clear(&self) {
        let ptr = self.map.load(core::sync::atomic::Ordering::Acquire);
        // SAFETY: The pointer is expected to be valid or `null`.
        let Some(ptr) = (unsafe { ptr.as_ref() }) else {
            return;
        };
        ptr.clear();
    }

    /// Initializes the interning set.
    #[inline]
    fn init(&self) -> &dashmap::DashMap<ByteData<'static>, ()> {
        let ptr = Arc::into_raw(Arc::new(dashmap::DashMap::<ByteData<'static>, ()>::new()))
            as *mut dashmap::DashMap<ByteData<'static>, ()>;
        match self.map.compare_exchange(
            core::ptr::null_mut(),
            ptr,
            core::sync::atomic::Ordering::AcqRel,
            core::sync::atomic::Ordering::Acquire,
        ) {
            // SAFETY: If the pointer is inserted, it is valid.
            Ok(_) => unsafe { &*ptr },
            Err(ret) => {
                // SAFETY: If there is a previous value, the new value should be dropped.
                core::mem::drop(unsafe { Arc::from_raw(ptr) });
                // SAFETY: The pointer is expected to be valid.
                unsafe { &*ret }
            }
        }
    }
}

impl Drop for StaticInterning {
    #[inline]
    fn drop(&mut self) {
        let ptr = self
            .map
            .swap(core::ptr::null_mut(), core::sync::atomic::Ordering::AcqRel);
        if ptr.is_null() {
            return;
        }
        // SAFETY: The pointer is expected to be a raw arc pointer.
        let ptr = unsafe { Arc::from_raw(ptr) };
        core::mem::drop(ptr);
    }
}

impl Default for StaticInterning {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new(128)
    }
}

impl Clone for StaticInterning {
    #[inline]
    fn clone(&self) -> Self {
        let ptr = self.map.load(core::sync::atomic::Ordering::Acquire);
        // SAFETY: The pointer is expected to be valid or `null`.
        let ptr = unsafe { ptr.as_ref() }.unwrap_or_else(|| self.init());
        // SAFETY: The pointer is valid and points to an `Arc`.
        unsafe { Arc::increment_strong_count(ptr) };
        Self {
            max_len: self.max_len,
            map: AtomicPtr::new(
                ptr as *const dashmap::DashMap<ByteData<'static>, ()>
                    as *mut dashmap::DashMap<ByteData<'static>, ()>,
            ),
        }
    }
}

impl core::fmt::Debug for StaticInterning {
    #[allow(clippy::missing_inline_in_public_items, clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let map = self.map.load(core::sync::atomic::Ordering::Acquire);
        let map =
            // SAFETY: The pointer is expected to be valid or `null`.
            unsafe { map.as_ref() }.map_or::<&dyn core::fmt::Debug, _>(&Option::<()>::None, |x| x);
        f.debug_struct("StaticInterning")
            .field("max_len", &self.max_len)
            .field("map", map)
            .finish()
    }
}

impl super::ByteInterning<'static> for StaticInterning {
    #[inline]
    fn intern<'b>(&self, value: ByteData<'b>) -> Result<ByteData<'static>, ByteData<'b>> {
        let len = value.len();
        if len <= crate::ByteChunk::LEN {
            return Ok(value.into_shared());
        }
        if len > self.max_len {
            return Err(value);
        }
        Ok(self.intern_always(value))
    }

    #[allow(single_use_lifetimes, clippy::needless_lifetimes)]
    #[inline]
    fn intern_always<'b>(&self, value: ByteData<'b>) -> ByteData<'static> {
        let ptr = self.map.load(core::sync::atomic::Ordering::Acquire);
        // SAFETY: The pointer is expected to be valid or `null`.
        let ptr = unsafe { ptr.as_ref() }.unwrap_or_else(|| self.init());
        if let Some(value) = ptr.get(value.as_slice()) {
            return value.key().clone();
        }
        let value = value.into_shared();
        match ptr.entry(value) {
            dashmap::Entry::Occupied(entry) => entry.key().clone(),
            dashmap::Entry::Vacant(entry) => entry.insert(()).key().clone(),
        }
    }

    #[inline]
    fn get<'b>(&self, value: ByteData<'b>) -> Result<ByteData<'static>, ByteData<'b>> {
        let ptr = self.map.load(core::sync::atomic::Ordering::Acquire);
        // SAFETY: The pointer is expected to be valid or `null`.
        let Some(ptr) = (unsafe { ptr.as_ref() }) else {
            return Err(value);
        };
        ptr.get(value.as_slice())
            .map_or_else(|| Err(value), |entry| Ok(entry.key().clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ByteInterning;

    #[test]
    fn test_static_interning_borrowed() {
        let interning = StaticInterning::new(32);
        let borrowed = ByteData::from_borrowed(b"Hello World, 1234567890!");
        let value = interning.intern(borrowed.clone()).unwrap();
        assert!(!core::ptr::addr_eq(value.as_slice().as_ptr(), borrowed.as_slice().as_ptr()), "The interned value should be a copy.");
        assert_eq!(value, borrowed, "The interned value should be equal to the input.");
        let value2 = interning.intern(borrowed.clone()).unwrap();
        assert_eq!(value2, borrowed, "The interned value should be equal to the input.");
        assert!(core::ptr::addr_eq(value.as_slice().as_ptr(), value2.as_slice().as_ptr()), "The interned value should return the same copy.");
    }

    #[test]
    fn test_static_interning_shared() {
        let interning = StaticInterning::new(32);
        let borrowed = ByteData::from_borrowed(b"Hello World, 1234567890!");
        let shared = borrowed.clone().into_shared();
        let value = interning.intern(shared.clone()).unwrap();
        assert!(core::ptr::addr_eq(value.as_slice().as_ptr(), shared.as_slice().as_ptr()), "The interned value should return a shared instance.");
        assert_eq!(value, borrowed, "The interned value should be equal to the input.");
        let value2 = interning.intern(borrowed.clone()).unwrap();
        assert_eq!(value2, borrowed, "The interned value should be equal to the input.");
        assert!(core::ptr::addr_eq(value.as_slice().as_ptr(), value2.as_slice().as_ptr()), "The interned value should return the same copy.");
    }
}

use std::ffi::c_void;

extern "C" {
    fn proxy_dyn_get_thread_context() -> *const c_void;
    fn proxy_dyn_set_limited_thread_context(thread_context: *const c_void);
}

#[derive(Clone, Copy)]
pub struct ThreadContext(*const c_void);

unsafe impl Send for ThreadContext {}
unsafe impl Sync for ThreadContext {}

impl ThreadContext {
    pub fn current() -> Option<Self> {
        let raw = unsafe { proxy_dyn_get_thread_context() };
        if raw.is_null() {
            return None;
        }
        Some(Self(raw))
    }

    pub fn enter(self) {
        unsafe { proxy_dyn_set_limited_thread_context(self.0) };
    }
}

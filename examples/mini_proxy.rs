use log::Level;
use proxy_sdk::{
    BaseContext, ConstCounter, Context, FilterDataStatus, HttpBodyControl, HttpContext,
    RequestBody, ResponseBody, RootContext,
};

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[global_allocator]
    static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

    #[no_mangle]
    pub extern "C" fn free(from: *mut std::ffi::c_void) {
        unsafe { drop(Box::from_raw(from as *mut u8)) };
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use core::alloc::{GlobalAlloc, Layout};

    #[global_allocator]
    static ALLOC: Mallocator = Mallocator;

    pub struct Mallocator;

    unsafe impl GlobalAlloc for Mallocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            malloc(layout.size())
        }

        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            free(ptr);
        }
    }

    extern "C" {
        fn malloc(size: usize) -> *mut u8;
        fn free(ptr: *mut u8);
    }
}

pub static FOUND_KEYWORD: ConstCounter = ConstCounter::define("proxy_found_hello_keyword");

#[derive(Default)]
pub struct ExampleContext {}

impl ExampleContext {
    const KEYWORD: &'static [u8] = b"hello";

    fn scan_for_regex(body: &impl HttpBodyControl) {
        if let Some(b) = body.all() {
            let n = b
                .windows(Self::KEYWORD.len())
                .filter(|w| *w == Self::KEYWORD)
                .count() as i64;
            FOUND_KEYWORD.get().increment(n);
        }
    }
}

impl BaseContext for ExampleContext {}

impl HttpContext for ExampleContext {
    fn on_http_request_body(&mut self, body: &RequestBody) -> FilterDataStatus {
        ExampleContext::scan_for_regex(body);
        FilterDataStatus::Continue
    }

    fn on_http_response_body(&mut self, body: &ResponseBody) -> FilterDataStatus {
        ExampleContext::scan_for_regex(body);
        FilterDataStatus::Continue
    }
}

#[derive(Default)]
pub struct ExampleRootContext {}

impl BaseContext for ExampleRootContext {}

impl RootContext for ExampleRootContext {
    fn create_context(&mut self) -> Context {
        Context::Http(Box::<ExampleContext>::default())
    }
}

fn init() {
    proxy_sdk::reset();
    proxy_sdk::set_log_level(Level::Trace);
    proxy_sdk::set_root_context_factory(ExampleRootContext::default);
}

#[no_mangle]
pub fn _start() {
    init();
}

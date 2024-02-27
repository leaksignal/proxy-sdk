use std::collections::HashMap;

use log::error;
use once_cell::sync::Lazy;

#[cfg(target_arch = "wasm32")]
fn read_environment() -> Result<Vec<(String, String)>, wasi::Errno> {
    let (count, size) = unsafe { wasi::environ_sizes_get()? };
    let mut entries: Vec<*mut u8> = Vec::with_capacity(count);

    let mut buf: Vec<u8> = Vec::with_capacity(size);
    unsafe { wasi::environ_get(entries.as_mut_ptr(), buf.as_mut_ptr())? };
    unsafe { entries.set_len(count) };
    // buf must never be accessed

    let mut out = Vec::new();
    for entry in entries {
        let cstr = unsafe { std::ffi::CStr::from_ptr(entry as *const i8) }.to_string_lossy();
        if let Some((name, value)) = cstr.split_once('=') {
            out.push((name.to_string(), value.to_string()));
        }
    }

    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_environment() -> Result<Vec<(String, String)>, std::convert::Infallible> {
    Ok(std::env::vars().collect::<Vec<_>>())
}

static ENV: Lazy<Vec<(String, String)>> = Lazy::new(|| match read_environment() {
    Ok(x) => x,
    Err(e) => {
        error!("failed to read environment: {e:?}");
        Default::default()
    }
});
static ENV_MAP: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| ENV.iter().map(|(k, v)| (&**k, &**v)).collect());

/// Get an environment variable value. Subject to whitelists and modifications from Envoy configuration.
pub fn var(name: impl AsRef<str>) -> Option<&'static str> {
    ENV_MAP.get(name.as_ref()).copied()
}

/// Get all environment variable values. Subject to whitelists and modifications from Envoy configuration.
pub fn vars() -> &'static HashMap<&'static str, &'static str> {
    &ENV_MAP
}

/// Get all environment variable values in original ordering. Subject to whitelists and modifications from Envoy configuration.
pub fn vars_ordered() -> &'static [(String, String)] {
    &ENV[..]
}

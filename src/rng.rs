use std::num::NonZeroU32;

fn proxywasm_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    if let Err(Some(e)) = unsafe { wasi::random_get(buf.as_mut_ptr(), buf.len()) }
        .map_err(|e| NonZeroU32::new(e.raw() as u32))
    {
        Err(e.into())
    } else {
        Ok(())
    }
}

getrandom::register_custom_getrandom!(proxywasm_getrandom);

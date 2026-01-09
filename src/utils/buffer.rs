/// Create a buffer with capacity and length set to desired size.
///
/// Is immediately addressable over the initial size.
pub unsafe fn uninitialized_buffer<T>(size: usize) -> Vec<T> {
    let mut buf = Vec::<T>::with_capacity(size);
    #[allow(clippy::uninit_vec)]
    unsafe {
        buf.set_len(size);
    };
    buf
}

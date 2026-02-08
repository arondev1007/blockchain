pub struct Memory;
impl Memory {
    #[unsafe(no_mangle)]
    fn mem_alloc(len: usize) -> *mut u8 {
        let mut buf = Vec::with_capacity(len);
        let ptr = buf.as_mut_ptr();

        std::mem::forget(buf);
        ptr
    }

    #[unsafe(no_mangle)]
    unsafe fn mem_dealloc(ptr: *mut u8, size: usize) {
        let data = Vec::from_raw_parts(ptr, size, size);
        std::mem::drop(data);
    }

    pub fn new(len: usize, data: &[u8]) -> *mut u8 {
        let mut buffer = Vec::with_capacity(4 + len);

        // data_len ( 4byte ) + data
        buffer.extend_from_slice(&len.to_le_bytes());
        buffer.extend_from_slice(data);

        let ptr = buffer.as_mut_ptr();
        std::mem::forget(buffer);

        ptr
    }

    pub fn export_length(ptr: *mut u8) -> usize {
        let input = unsafe { std::slice::from_raw_parts(ptr, 4) };
        let input_len = u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize;

        input_len
    }

    // -----------------------------------------------------------------------------------------
    // Memory write / read

    pub fn encode(bytes: &[u8]) -> Vec<u8> {
        let len_val = bytes.len() as u32;
        let mut buffer = Vec::with_capacity(4 + len_val as usize);

        // len ( 4byte ) + val
        buffer.extend_from_slice(&len_val.to_le_bytes());
        buffer.extend_from_slice(bytes);

        buffer
    }

    pub fn decode(encoded: &[u8]) -> Vec<u8> {
        if encoded.len() < 4 {
            return Vec::new();
        }

        let input_len =
            u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]) as usize;

        if encoded.len() < 4 + input_len {
            return Vec::new();
        }

        encoded[4..4 + input_len].to_vec()
    }

    pub fn decode_ptr(ptr: *mut u8) -> Vec<u8> {
        let size = Memory::export_length(ptr);
        let data = unsafe { std::slice::from_raw_parts(ptr.add(4), size) };

        data.to_vec()
    }

    pub fn decode_len(len: &[u8]) -> usize {
        if len.len() != 4 {
            return 0;
        }

        let input_len = u32::from_le_bytes([len[0], len[1], len[2], len[3]]) as usize;
        input_len
    }
}

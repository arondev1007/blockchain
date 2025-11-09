use memory::Memory;

#[unsafe(no_mangle)]
pub fn example() -> *mut u8 {
    let dummy = vec![0, 100, 250, 0, 1, 2];
    let ptr = Memory::new(dummy.len(), &dummy);

    return ptr;
}

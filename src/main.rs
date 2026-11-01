#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// `libc::mallopt` has no safe Rust binding. Raising `M_MMAP_THRESHOLD`
// makes glibc route large allocations to mmap instead of retaining them in
// the heap, reducing fragmentation. This is the sole `unsafe` block in the
// binary crate; every other module should remain `unsafe_code`-free.
#[allow(unsafe_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prevents glibc from hoarding memory via memory fragmentation.
    #[cfg(all(not(feature = "jemalloc"), target_env = "gnu"))]
    unsafe {
        libc::mallopt(libc::M_MMAP_THRESHOLD, 65536);
    }

    commander::main()
}

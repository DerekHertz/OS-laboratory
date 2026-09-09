//! Minimal Wasm ABI probe, not the future simulation transport protocol.

#[unsafe(no_mangle)]
pub extern "C" fn build_probe(left: u32, right: u32) -> u32 {
    sim_core::build_probe(left, right)
}

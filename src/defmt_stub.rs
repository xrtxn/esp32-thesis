#[cfg(feature = "defmt")]
#[defmt::timestamp]
fn timestamp() -> u64 {
    embassy_time::Instant::now().as_ticks()
}

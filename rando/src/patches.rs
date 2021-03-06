use lazy_static::lazy_static;

lazy_static! {
    pub static ref PATCHES: Vec<&'static [u8]> = vec![
        include_bytes!(concat!(env!("OUT_DIR"), "/asm/expand-save-state.ips")),
        include_bytes!(concat!(env!("OUT_DIR"), "/asm/intro-skip.ips")),
        include_bytes!(concat!(env!("OUT_DIR"), "/asm/no-downgrade.ips")),
        include_bytes!(concat!(env!("OUT_DIR"), "/asm/open-stairs.ips")),
        include_bytes!(concat!(env!("OUT_DIR"), "/asm/text-speedup.ips")),
    ];
}

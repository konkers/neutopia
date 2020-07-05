use wasm_bindgen::prelude::*;

use radix_fmt::radix_36;
use rand::{self, prelude::*};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;

extern crate console_error_panic_hook;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
#[wasm_bindgen]
pub struct RandomizedRom {
    rom: Vec<u8>,
    filename: String,
}

#[wasm_bindgen]
impl RandomizedRom {
    pub fn get_rom(&self) -> Vec<u8> {
        self.rom.clone()
    }
    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }
}

#[wasm_bindgen]
pub fn randomize_rom(data: &[u8], seed_str: &str) -> RandomizedRom {
    console_error_panic_hook::set_once();

    log!("got data length {} and seed {}", data.len(), seed_str);
    let seed: u64;
    if seed_str.is_empty() {
        seed = rand::thread_rng().gen();
    } else if let Ok(parsed_seed) = u64::from_str_radix(seed_str, 36) {
        seed = parsed_seed;
    } else {
        seed = rand::thread_rng().gen();
    }

    let mut rng = Pcg32::seed_from_u64(seed);

    let mut buffer = Vec::new();
    buffer.extend_from_slice(data);

    if let Err(err) = randolib::randomize_rom(&mut buffer, &mut rng) {
        log!("Got error {}", err);
    }

    let filename = format!("neutopia-randomizer-{:#}.pce", radix_36(seed));

    log!("wrote {}", filename);

    RandomizedRom {
        rom: buffer,
        filename,
    }
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

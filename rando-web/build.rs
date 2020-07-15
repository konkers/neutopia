use vergen::{generate_cargo_keys, ConstantsFlags};

fn main() {
    // Generate version, build date, and sha.
    let mut flags = ConstantsFlags::all();
    flags.toggle(ConstantsFlags::SEMVER_FROM_CARGO_PKG);
    generate_cargo_keys(flags).expect("Unable to generate the cargo keys!");
}

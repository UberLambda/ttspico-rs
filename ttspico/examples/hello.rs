use ttspico as pico;

fn main() {
    let sys = pico::System::new(512 * 1024)
        .unwrap_or_else(|err| panic!("Could not init system: {}", err));
}

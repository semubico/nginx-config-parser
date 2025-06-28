use nginx_config_parser::{Token, Structure};

fn main() {
    let cfg = std::env::args().nth(1).unwrap();
    let cfg = std::fs::read_to_string(&cfg).unwrap();
    let cfg = Structure::parse(&cfg);

    dbg!(cfg);
}

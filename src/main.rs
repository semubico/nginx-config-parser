#![allow(unused)]
use nginx_config_parser::{Token, Structure};

// TBD
// fn print_directive(cfg: &Structure) {
//     match cfg {
//         Structure::Statement { args, .. } => {
//             let args = args.iter().map(|s| format!("{}", s.to_string())).collect::<String>();
//             println!("{args}");
//         },
//         Structure::Block { args, children, .. } => {
//             let args = args.iter().map(|s| format!("{}", s.to_string())).collect::<String>();
//             for each in children.iter() {
//                 print_directive(&each);
//             }
//         }
//     }
// }

fn main() {

    let cfg = std::env::args().nth(1).unwrap();   
    let cfg = std::fs::read_to_string(&cfg).unwrap();
    let cfg = Structure::parse(&cfg).unwrap();

    dbg!(cfg);
}

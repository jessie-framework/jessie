use std::env;

mod create_new_project;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_, command, argument] if command == "new" => {
            create_new_project(argument);
        }
        _ => {}
    }
}

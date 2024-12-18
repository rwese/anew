mod app;

use std::io;

fn main() {
    let args = app::args();

    let stdio = io::stdin();
    let input = stdio.lock();

    let output = io::stdout();

    let result = app::app(args, output, input);
    match result {
        Ok(_) => {}
        Err(err) => panic!("{}", err),
    }
}


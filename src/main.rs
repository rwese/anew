mod app;

use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = app::args();

    let stdio = io::stdin();
    let input = stdio.lock();

    let output = io::stdout();

    app::app(args, output, input)?;

    Ok(())
}

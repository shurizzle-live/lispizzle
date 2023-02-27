use lispizzle::{parser::parse_from_file, Environment};

extern crate lispizzle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Environment::default();
    let code = parse_from_file("examples/1.zle".as_ref())?;
    for exp in code {
        if let Err(err) = exp.eval(env.clone()) {
            println!("{:#?}", err);
            return Ok(());
        }
    }
    Ok(())
}

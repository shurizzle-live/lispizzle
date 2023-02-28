use lispizzle::{
    parser::{parse_from_file, FileParseError},
    Environment,
};

extern crate lispizzle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code = match parse_from_file("examples/1.zle".as_ref()) {
        Ok(x) => x,
        Err(FileParseError::Parse(err)) => {
            println!("{}", err);
            return Ok(());
        }
        e => e?,
    };

    let env = Environment::default();

    for exp in code {
        if let Err(err) = exp.eval(env.clone()) {
            println!("{:#?}", err);
            return Ok(());
        }
    }
    Ok(())
}

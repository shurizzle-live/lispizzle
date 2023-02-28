use lispizzle::{
    parser::{parse_from_file, FileParseError},
    Environment, Program,
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

    let prog = match Program::new(code) {
        Ok(x) => x,
        Err(err) => {
            println!("{:#?}", err);
            return Ok(());
        }
    };

    let env = Environment::default();

    match prog.eval(env) {
        Ok(_) => (),
        Err(err) => {
            println!("{:#?}", err);
            return Ok(());
        }
    }

    Ok(())
}

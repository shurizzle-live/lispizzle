use lispizzle::{
    parser::{parse_from_file, FileParseError},
    BackTrace, Environment, Program,
};

extern crate lispizzle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code = match parse_from_file("examples/macros.zle".as_ref()) {
        Ok(x) => x,
        Err(FileParseError::Parse(err)) => {
            println!("{}", err);
            return Ok(());
        }
        e => e?,
    };

    let prog = Program::new(code);

    match prog.eval(BackTrace::new(), Environment::default()) {
        Ok(_) => (),
        Err(err) => {
            println!("{:#?}", err);
            return Ok(());
        }
    }

    Ok(())
}

use lispizzle::{
    parser::{parse_from_file_with_cache, FileParseError},
    Context, Environment, Program, StrCache,
};

extern crate lispizzle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = StrCache::new();

    let code = match parse_from_file_with_cache("examples/macros.zle".as_ref(), cache.clone()) {
        Ok(x) => x,
        Err(FileParseError::Parse(err)) => {
            println!("{}", err);
            return Ok(());
        }
        e => e?,
    };

    let prog = Program::new(code);

    match prog.eval(Context::with_cache(cache), Environment::default()) {
        Ok(_) => (),
        Err(err) => {
            println!("{:#?}", err);
            for frame in err.backtrace().into_iter() {
                println!("{:?}", frame);
            }
            return Ok(());
        }
    }

    Ok(())
}

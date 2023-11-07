use std::{fs::File, io::Write, path::Path};

use clap::Parser;
use code::macros::MacroExpander;
use errors::report_error;
use runtime::RuntimeError;

use crate::code::{ast_visitor::Visitor, builder::Builder};

#[cfg(test)]
#[macro_use]
extern crate spectral;

mod code;
mod errors;
mod lua;
mod parser;
pub mod runtime;
#[cfg(test)]
mod tests;

#[derive(Parser, Clone)]
struct Args {
    #[arg(
        short,
        long,
        help = "An optional output file, if not provided the extension is replaced by .lua"
    )]
    output: Option<String>,
    #[arg(help = "The input file to evaluate and/or compile")]
    input: String,
    #[arg(short, long, help = "Compiles the Saturnus script")]
    compile: bool,
    #[arg(
        short = 'p',
        long = "print",
        help = "Prints the compilation result to the standard output"
    )]
    print: bool,
    #[arg(
        long,
        help = "If used, the compilation output emits tab characters. Ignores indentation parameter"
    )]
    use_tabs: bool,
    #[arg(
        default_value = "2",
        long,
        help = "The amount of space characters to use in each tab"
    )]
    indent: usize,
    #[arg(long, help = "Strips the std library form the code emission")]
    no_std: bool,
    #[arg(long, help = "Inline the std library in each script")]
    inline_std: bool,
    #[arg(
        long,
        help = "Outputs the saturnus code to stdout preprocessed but without compiling"
    )]
    dump_saturnus: bool,
}

fn get_default_output(str: &Path) -> String {
    Path::new(str)
        .with_extension("lua")
        .to_str()
        .unwrap()
        .to_string()
}

struct CompilationOptions {
    args: Args,
    in_path: String,
    out_path: String,
}

fn try_run(options: CompilationOptions, input: String, indent: String) -> Result<(), RuntimeError> {
    // TODO: Clean std code injection
    let header = format!("let __modules__ = {{ }};");
    let header = if options.args.no_std {
        header
    } else {
        let embed = include_str!("assets/std.saturn");
        format!("{header}\n__modules__.std = {{\n{embed}\n}};")
    };

    let compiler = lua::visitor::LuaEmitter::new();

    if options.args.dump_saturnus {
        println!("{input}");
        return Ok(());
    }

    let mut macro_expander = MacroExpander::new();

    let script = parser::Script::parse(format!("{header}\n{input}"))
        .map_err(|err| RuntimeError::ParseError(err))?;
    let script = macro_expander.compile_macros(&script).unwrap();
    let script = macro_expander.expand_macros(&script).unwrap();

    let CompilationOptions {
        args,
        out_path,
        in_path,
    } = options;

    if args.compile {
        println!("Compiling {:?}...", in_path);
        let output = compiler
            .visit_script(Builder::new(indent), &script)
            .map_err(|err| RuntimeError::CompilationError(err))?
            .collect();
        if args.print {
            println!("\n------\n\n");
            std::io::stdout().write_all(output.as_bytes()).unwrap();
        } else {
            let mut out_file = File::create(out_path).unwrap();
            out_file.write_all(output.as_bytes()).unwrap();
        }
    } else {
        let host: runtime::RuntimeHost =
            runtime::RuntimeHost::new(indent.clone(), Box::new(compiler));
        host.evaluate(&script)?;
    }

    Ok(())
}

fn main() {
    // Configure environment
    let args = Args::parse();
    let indent = if args.use_tabs {
        "\t".to_string()
    } else {
        " ".repeat(args.indent)
    };
    use std::fs::read_to_string;

    // Read input files
    let in_path = Path::new(&args.input);
    let out_path = args.clone().output.unwrap_or(get_default_output(in_path));
    let input = read_to_string(in_path).unwrap();

    let options = CompilationOptions {
        args: args.clone(),
        in_path: in_path.to_str().unwrap().to_owned(),
        out_path: out_path.to_owned(),
    };
    match try_run(options, input.clone(), indent.clone()) {
        Ok(_) => (),
        Err(err) => match err {
            RuntimeError::EvaluationError(err) => eprintln!("{}", err),
            RuntimeError::ParseError(err) => {
                let err = report_error(args.input.clone(), input.clone(), err);
                if args.compile && !args.print {
                    let mut out_file = File::create(out_path).unwrap();
                    let output = format!("error[=====[{}]=====]", err);
                    out_file.write_all(output.as_bytes()).unwrap();
                }
                eprintln!("{}\nCompilation failed", err);
            }
            RuntimeError::CompilationError(err) => eprintln!("{:?}", err),
        },
    }
}

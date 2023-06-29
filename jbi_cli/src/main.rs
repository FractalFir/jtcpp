use clap::Parser;
use std::fs::File;
use jbi::*;
#[derive(Parser,Debug)]
struct ExecutionArgs{
    #[arg(short = 'm', long = "main-class")]
    main_class:Option<String>,
    //#[arg(short = 'p', long = "file-path")]
    src_path: std::path::PathBuf,
    //print_deps:bool
}
fn main() {
    let args = ExecutionArgs::parse();
    let src_path = &args.src_path;
    let src_file = File::open(src_path);
    let mut src_file = match src_file{
        Ok(src_file)=>src_file,
        Err(err)=>{
            eprintln!("Could not read file at {src_path:?}, because {err:?}!");
            return;
        },
    };
    let extension = if let Some(extension) = src_path.extension(){
        extension
    }
    else{
        eprintln!("Could not read file at {src_path:?}, because it has no extension, so it's type could not be determined!");
        return;
    };
    let mut exec_env = ExecEnv::new();
    exec_env.insert_stdlib();
    match extension.to_str(){
        Some("jar")=>exec_env.import_jar(&mut src_file).unwrap(),
        Some("class")=>exec_env.import_class(&mut src_file).unwrap(),
        _=>{
            eprintln!("Could not import file at {src_path:?}, because it's extension {extension:?}, is neither .jar nor .class, so executing it is not supported.");
            return;
        }
    }
    let main = exec_env.try_find_main(args.main_class.as_deref());
    let main = match main{
        Some(main)=>main,
        None=>{
            eprintln!("Could run the speciifed file, because it has no main method.");
            return;
        }
    };
    let result = exec_env.call_method(main,&[Value::ObjectRef(0)]).expect("Exception!");
    //println!("args:{args:?}");
}

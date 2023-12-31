mod class;
mod cpp_codegen;
mod fatops;
mod importer;
mod method;
use crate::fatops::{ClassInfo, FatOp};
use crate::importer::{BytecodeImportError, ImportedJavaClass};
use clap::Parser;
use include_dir::{include_dir, Dir};
use std::{io::Write, path::PathBuf};
use {class::Class, method::Method};
pub type IString = Box<str>;
static STDLIB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/stdlib");
static GC_CPP_HEADER: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/bdwgc/include/gc/gc_cpp.h"));
static GC_HEADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bdwgc/include/gc/gc.h"));
static GC_VERSION_HEADER: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/bdwgc/include/gc/gc_version.h"));
static GC_CONFIG_MACROS: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/bdwgc/include/gc/gc_config_macros.h"
));
static GC_SO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bdwgc/out/libgc.so"));
static GCCPP_SO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bdwgc/out/libgccpp.so"));

fn method_name_to_c_name(method_name: &str) -> IString {
    match method_name {
        "<init>" => "_init_".into(),
        "<clinit>" => "_clinit_".into(),
        _ => method_name.into(),
    }
}
fn class_path_to_class_mangled(class_path: &str) -> IString {
    let mut out = String::with_capacity(class_path.len());
    let mut sequences = class_path.split('/');
    if let Some(prefix) = sequences.next() {
        out.push_str(prefix)
    };
    for seq in sequences {
        out.push_str("_cs_");
        out.push_str(seq)
    }
    let out = out.replace('$', "_dolsig_");
    out.into()
}
#[test]
fn desc_mangling() {
    assert_eq!(&*desc_to_mangled("(D)D"), "_D_D");
}
fn desc_to_mangled(desc: &str) -> IString {
    let mut classname_beg = 0;
    let mut within_class = false;
    let mut res = String::new();
    for (index, curr) in desc.chars().enumerate() {
        if curr == 'L' {
            within_class = true;
            classname_beg = index + 1;
        }
        if curr == ';' {
            within_class = false;
            let class = &desc[classname_beg..index];
            let class = class_path_to_class_mangled(class);
            res.push_str(&class);
            res.push_str("_");
            continue;
        }
        if curr == '(' {
            res.push_str("_");
            continue;
        } else if curr == ')' {
            res.push_str("_");
            continue;
        }
        if !within_class {
            res.push(curr);
        }
    }
    res.replace('[', "_arr_").into()
}
fn mangle_method_name(method: &str, desc: &str) -> IString {
    let desc = desc_to_mangled(desc);
    let method = method_name_to_c_name(method);
    format!("{method}{desc}").into()
}
#[derive(Debug, Clone, PartialEq)]
enum VariableType {
    Void,
    Char,
    Bool,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ObjectRef(ClassInfo),
    ArrayRef(Box<VariableType>),
}
impl VariableType {
    pub(crate) fn dependency(&self) -> Option<IString> {
        match self {
            Self::Void
            | Self::Char
            | Self::Bool
            | Self::Byte
            | Self::Short
            | Self::Int
            | Self::Long
            | Self::Float
            | Self::Double => None,
            Self::ObjectRef(class_info) => Some(class_info.class_path()),
            Self::ArrayRef(var) => var.dependency(),
        }
    }
    fn is_array(&self) -> bool {
        matches!(self, Self::ArrayRef(_))
    }
    fn is_wide(&self) -> bool {
        matches!(self, Self::Long | Self::Double)
    }
    fn assignable(&self, other: &Self) -> bool {
        match self {
            Self::Float => matches!(other, Self::Double | Self::Float),
            Self::Double => matches!(other, Self::Double | Self::Float),
            Self::Long => matches!(other, Self::Long | Self::Int | Self::Byte | Self::Short),
            Self::Int => matches!(other, Self::Int | Self::Byte | Self::Short),
            Self::Bool => matches!(other, Self::Long | Self::Int | Self::Byte | Self::Short),
            Self::Byte => matches!(other, Self::Long | Self::Int | Self::Byte | Self::Short),
            Self::Short => matches!(other, Self::Long | Self::Int | Self::Byte | Self::Short),
            Self::Char => matches!(other, Self::Long | Self::Int | Self::Byte | Self::Short),
            Self::Void => matches!(other, Self::Long | Self::Int | Self::Byte | Self::Short),
            Self::ObjectRef(_) => true,
            Self::ArrayRef(_) => true,
        }
    }
    fn c_type(&self) -> IString {
        match self {
            Self::Float => "float".into(),
            Self::Double => "double".into(),
            Self::Long => "long".into(),
            Self::Int => "int32_t".into(),
            Self::Bool => "bool".into(),
            Self::Byte => "int8_t".into(),
            Self::Short => "int16_t".into(),
            Self::Char => "char16_t".into(),
            Self::Void => "void".into(),
            Self::ObjectRef(info) => format!("ManagedPointer<{}>", info.cpp_class()).into(),
            Self::ArrayRef(atype) => {
                format!("ManagedPointer<RuntimeArray<{}>>", atype.c_type()).into()
            } //_=>todo!("Can't get ctype of {self:?}!"),
        }
    }
    fn unknown()->Self{
        Self::ObjectRef(ClassInfo::unknown()) 
    }
    fn is_unknown(&self) -> bool {
        if let Self::ObjectRef(class_info) = self {
            class_info.is_unknown()
        } else {
            false
        }
    }
    fn type_postifx(&self) -> IString {
        match self {
            Self::Float => "f".into(),
            Self::Double => "d".into(),
            Self::Long => "l".into(),
            Self::Int => "i".into(),
            Self::Bool => "z".into(),
            Self::Byte => "b".into(),
            Self::Short => "s".into(),
            Self::ObjectRef(_) => "a".into(),
            Self::ArrayRef(_atype) => "a".into(),
            _ => todo!("Can't get type postifx of {self:?}!"),
        }
    }
}
#[test]
fn nested_array_desc_to_ftype() {
    assert_eq!(
        field_desc_str_to_ftype("[[I", 0),
        VariableType::ArrayRef(Box::new(VariableType::ArrayRef(Box::new(
            VariableType::Int
        ))))
    );
}
pub(crate) fn field_desc_str_to_ftype(desc_str: &str, th: usize) -> VariableType {
    let beg = desc_str.chars().nth(th).unwrap();
    match beg {
        'B' => VariableType::Byte,
        'C' => VariableType::Char,
        'D' => VariableType::Double,
        'F' => VariableType::Float,
        'I' => VariableType::Int,
        'J' => VariableType::Long,
        'L' => VariableType::ObjectRef(ClassInfo::from_java_path(
            desc_str[(th + 1)..(desc_str.len() - 1)]
                .split(';')
                .next()
                .unwrap(),
        )),
        '[' => VariableType::ArrayRef(Box::new(field_desc_str_to_ftype(desc_str, th + 1))),
        'S' => VariableType::Short,
        'Z' => VariableType::Bool,
        'V' => VariableType::Void,
        _ => panic!("Invalid field descriptor!\"{desc_str}\". beg:{beg}"),
    }
}
pub(crate) fn field_descriptor_to_ftype(
    descriptor: u16,
    class: &ImportedJavaClass,
) -> VariableType {
    field_desc_str_to_ftype(class.lookup_utf8(descriptor).unwrap(), 0)
}
fn method_desc_to_args(desc: &str) -> (Vec<VariableType>, VariableType) {
    let arg_beg = desc.chars().position(|c| c == '(').unwrap() + 1;
    let arg_end = desc.chars().position(|c| c == ')').unwrap();
    let arg_desc = &desc[arg_beg..arg_end];
    let ret_val = field_desc_str_to_ftype(desc, arg_end + 1);
    let mut within_class = false;
    let mut args = Vec::new();
    for (index, curr) in arg_desc.chars().enumerate() {
        if !within_class {
            args.push(field_desc_str_to_ftype(arg_desc, index));
        }
        if curr == 'L' || curr == '[' {
            within_class = true;
        }
        if curr == ';' {
            within_class = false;
        }
    }
    (args, ret_val)
}
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct ConvertionArgs {
    // Source files to load and convert to C.
    #[arg(short, long)]
    source_files: Vec<PathBuf>,
    // Target directory
    #[arg(short, long)]
    out: PathBuf,
}
struct CompilationContext {}
const ERR_NO_EXT: i32 = 1;
const ERR_BAD_EXT: i32 = 2;
const ERR_FOPEN_FAIL: i32 = 3;
const ERR_THIS_INVALID: i32 = 4;
const ERR_SUPER_INVALID: i32 = 5;
const ERR_BAD_OUT: i32 = 6;
const ERR_HEADER_IO_FAIL: i32 = 7;
const PROGRESS_BAR_SIZE: usize = 50;
fn print_progress(curr: usize, whole: usize) {
    print!("\r{curr}/{whole} \t");
    let fract = ((curr as f64 / whole as f64) * (PROGRESS_BAR_SIZE as f64)).round() as usize;
    for i in 0..PROGRESS_BAR_SIZE {
        if i < fract {
            print!("█");
        } else {
            print!("░");
        }
    }
    std::io::stdout().flush().unwrap();
}
macro_rules! write_cpp_file {
    ($bytes:ident,$target_path:ident,$name:literal) => {{
        let mut target_path = $target_path.clone();
        target_path.push($name);
        if !target_path.exists() {
            let mut target_out = std::fs::File::create(target_path)?;
            target_out.write_all($bytes)?;
        }
    }};
}
impl CompilationContext {
    fn write_stdlib(target_path: &PathBuf) -> std::io::Result<()> {
        for file in STDLIB_DIR.files() {
            let mut target_path = target_path.clone();
            target_path.extend(file.path());
            if !target_path.exists() {
                let mut target_out = std::fs::File::create(target_path)?;
                target_out.write_all(file.contents())?;
            }
        }
        write_cpp_file!(GC_CPP_HEADER, target_path, "gc_cpp.h");
        write_cpp_file!(GC_HEADER, target_path, "gc.h");
        write_cpp_file!(GC_VERSION_HEADER, target_path, "gc_version.h");
        write_cpp_file!(GC_CONFIG_MACROS, target_path, "gc_config_macros.h");
        let mut build_path = target_path.clone();
        build_path.push("build");
        std::fs::create_dir_all(&build_path)?;
        write_cpp_file!(GC_SO, build_path, "libgc.so");
        write_cpp_file!(GCCPP_SO, build_path, "libgccpp.so");
        Ok(())
    }
    fn new(ca: &ConvertionArgs) -> Result<(), BytecodeImportError> {
        let mut loaded_classes = Vec::new();
        for (index, path) in ca.source_files.iter().enumerate() {
            let path_disp = path.display();
            let extension = path.extension();
            print_progress(index, ca.source_files.len());
            let extension = match extension {
                Some(extension) => extension,
                None => {
                    eprintln!("\nFile at {path_disp} has no extension, so it can't be determied if it is either .class or .jar, and can't be compiled!");
                    std::process::exit(ERR_NO_EXT);
                }
            };
            match extension.to_str() {
                Some("jar") => {
                    let mut src = match std::fs::File::open(path) {
                        Ok(src) => src,
                        Err(err) => {
                            eprintln!("\nFile at {path_disp} can't be opened because {err:?}!");
                            std::process::exit(ERR_FOPEN_FAIL);
                        }
                    };
                    let classes = importer::load_jar(&mut src)?;
                    loaded_classes.extend(classes);
                }
                Some("class") => {
                    let mut src = match std::fs::File::open(path) {
                        Ok(src) => src,
                        Err(err) => {
                            eprintln!("\nFile at {path_disp} can't be opened because {err:?}!");
                            std::process::exit(ERR_FOPEN_FAIL);
                        }
                    };
                    let class = importer::load_class(&mut src)?;
                    loaded_classes.push(class);
                }
                _ => {
                    eprintln!(
                        "\nfile at {path_disp} is neither .class nor .jar, and can't be compiled!"
                    );
                    std::process::exit(ERR_BAD_EXT);
                }
            };
            println!("\rSuccessfully loaded file {path_disp}!                           ");
        }
        println!("\r Finished stage 1(Import) of JVM bytecode to C++ translation.");
        let mut classes = Vec::with_capacity(loaded_classes.len());
        for (index, class) in loaded_classes.iter().enumerate() {
            print_progress(index, loaded_classes.len());
            classes.push(Class::from_java_class(class));
        }
        println!("\r Finished stage 2(Conversion) of JVM bytecode to C++ translation.");
        std::fs::create_dir_all(&ca.out).unwrap();
        Self::write_stdlib(&ca.out).unwrap();
        for (index, class) in classes.iter().enumerate() {
            print_progress(index, classes.len());
            let mut path = ca.out.clone();
            path.push(&*class.path());
            path.set_extension("hpp");
            let hout = std::fs::File::create(&path);
            let mut hout = match hout {
                Ok(hout) => hout,
                Err(_err) => {
                    eprintln!("\nCan't create file at {path}!", path = path.display());
                    std::process::exit(ERR_BAD_OUT);
                }
            };
            match cpp_codegen::create_header(&mut hout, class) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!(
                        "\nCan't write header at path{path}, beacuse {err:?}!",
                        path = path.display()
                    );
                    std::process::exit(ERR_HEADER_IO_FAIL);
                }
            }
            println!(
                "\rcreating file at path:{}                                        ",
                path.display()
            );
        }
        println!("\r Finished stage 3(Generating headers) of JVM bytecode to C++ translation.");
        for (index, class) in classes.iter().enumerate() {
            print_progress(index, classes.len());
            let mut path = ca.out.clone();
            path.push(&format!("{}", class.path()));
            path.set_extension("cpp");
            let mut class_cpp_out = std::fs::File::create(path)?;
            for (_, smethod) in class.static_methods() {
                cpp_codegen::create_method_impl(&mut class_cpp_out, smethod)?;
                writeln!(class_cpp_out)?;
            }
            for (_, smethod) in class.virtual_methods() {
                cpp_codegen::create_method_impl(&mut class_cpp_out, smethod)?;
                writeln!(class_cpp_out)?;
            }
            write!(
                class_cpp_out,
                "\n#include \"{class_path}.hpp\"\n",
                class_path = class.path()
            )?;
            for (static_name, static_type) in class.static_fields() {
                write!(
                    class_cpp_out,
                    "{ctype} {class_path}::{static_name};",
                    ctype = static_type.c_type(),
                    class_path = class.path()
                )?;
            }
        }
        println!(
            "\r Finished stage 4(Generating Source files) of JVM bytecode to C++ translation."
        );
        println!(
            "DONE! go to the target directory {target}, and run `make` to build your converted code.", target = ca.out.display(),
        );
        Ok(())
    }
}
fn main() {
    let args = ConvertionArgs::parse();
    CompilationContext::new(&args).unwrap();
}

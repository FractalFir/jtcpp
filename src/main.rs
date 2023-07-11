mod basic_block;
mod class;
mod cpp_codegen;
mod fatops;
mod importer;
mod method;
use class::Class;
use method::Method;
macro_rules! include_stdlib_header_file {
    ($file_name:ident) => {
        const $file_name: &[u8] =
            include_bytes!(concat!("../stdlib/", stringify!($file_name), ".hpp"));
    };
}
macro_rules! include_stdlib_source_file {
    ($file_name:ident) => {
        const $file_name: &[u8] =
            include_bytes!(concat!("../stdlib/", stringify!($file_name), ".cpp"));
    };
}
include_stdlib_header_file!(java_cs_lang_cs_Object);
include_stdlib_header_file!(java_cs_lang_cs_String);
include_stdlib_header_file!(runtime);
use crate::fatops::FatOp;
use crate::importer::{BytecodeImportError, ImportedJavaClass};
use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
pub type IString = Box<str>;
use basic_block::BasicBlock;
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
    match sequences.next() {
        Some(prefix) => out.push_str(prefix),
        None => (),
    }
    for seq in sequences {
        out.push_str("_cs_");
        out.push_str(seq)
    }
    let out = out.replace('$', "_dolsig_");
    out.into()
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
            res.push_str("_as_");
            continue;
        }
        if curr == '(' {
            res.push_str("_ab_");
            continue;
        } else if curr == ')' {
            res.push_str("ae_");
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
    format!("{method}_ne_{desc}").into()
}
fn mangle_method_name_partial(method: &str, desc: &str) -> IString {
    let desc = desc_to_mangled(desc);
    let method = method_name_to_c_name(method);
    format!("{method}_ne_{desc}").into()
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
    ObjectRef { name: IString },
    ArrayRef(Box<VariableType>),
}
impl VariableType {
    pub(crate) fn dependency(&self) -> Option<&str> {
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
            Self::ObjectRef { name } => Some(&name),
            Self::ArrayRef(var) => var.dependency(),
        }
    }
}
impl VariableType {
    fn c_type(&self) -> IString {
        match self {
            Self::Float => "float".into(),
            Self::Double => "double".into(),
            Self::Long => "long".into(),
            Self::Int => "int".into(),
            Self::Bool => "bool".into(),
            Self::Byte => "char".into(),
            Self::Short => "short".into(),
            Self::Char => "short".into(),
            Self::Void => "void".into(),
            Self::ObjectRef { name } => format!("{name}*").into(),
            Self::ArrayRef(atype) => format!("RuntimeArray<{}>*", atype.c_type()).into(),
            //_=>todo!("Can't get ctype of {self:?}!"),
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
            Self::ObjectRef { name: _ } => "a".into(),
            Self::ArrayRef(_atype) => "aa".into(),
            _ => todo!("Can't get type postifx of {self:?}!"),
        }
    }
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
        'L' => VariableType::ObjectRef {
            name: class_path_to_class_mangled(
                desc_str[(th + 1)..(desc_str.len() - 1)]
                    .split(';')
                    .next()
                    .unwrap()
                    .into(),
            ),
        },
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
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    field_desc_str_to_ftype(descriptor, 0)
}
#[test]
fn arg_counter() {
    assert_eq!(method_desc_to_argc("()I"), 0);
    assert_eq!(method_desc_to_argc("(I)I"), 1);
    assert_eq!(method_desc_to_argc("(IL)I"), 2);
    assert_eq!(method_desc_to_argc("(IJF)I"), 3);
    assert_eq!(method_desc_to_argc("(IJF)"), 3);
    assert_eq!(method_desc_to_argc("(Ljava/lang/Object;)V"), 1);
    assert_eq!(method_desc_to_argc("([[[D)V"), 1);
}
fn method_desc_to_argc(desc: &str) -> u8 {
    assert_eq!(desc.chars().nth(0), Some('('));
    let mut char_beg = 0;
    let mut char_end = 0;
    for (index, character) in desc.chars().enumerate() {
        if character == '(' {
            assert_eq!(char_beg, 0);
            char_beg = index;
        } else if character == ')' {
            assert_eq!(char_end, 0);
            char_end = index;
        }
    }
    let span = &desc[(char_beg + 1)..char_end];
    let mut res = 0;
    let mut ident = false;
    for curr in span.chars() {
        if ident {
            if matches!(curr, ';') {
                ident = false;
            }
            continue;
        } else if curr == 'L' {
            ident = true;
        } else if curr == '[' {
            continue;
        }
        res += 1;
    }
    //println!("span:{span},res{res}");
    res as u8
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
    println!("desc:{desc:?} args:{args:?}");
    (args, ret_val)
}

use std::collections::HashSet;
struct MethodCG {
    includes: String,
    class_name: IString,
    fn_name: IString,
    signature: IString,
    local_dec: String,
    locals: HashSet<IString>,
    im_idx: usize,
    code: String,
}
impl MethodCG {
    fn add_include(&mut self, file: &str) {
        //TODO:Handle double includes!
        self.includes.push_str("#include \"");
        self.includes.push_str(file);
        self.includes.push_str(".hpp\"\n");
    }
    fn ensure_exists(&mut self, varname: &str, vartype: &VariableType) {
        let varname: IString =
            format!("{varname}{postfix}", postfix = vartype.type_postifx()).into();
        if !self.locals.contains(&varname) {
            let ctype = vartype.c_type();
            self.local_dec.push_str(&format!("\t{ctype} {varname}"));
            match vartype {
                VariableType::ArrayRef(_) => {
                    self.local_dec.push_str("(nullptr);\n");
                }
                VariableType::ObjectRef { name: _ } => {
                    self.local_dec.push_str("(nullptr);\n");
                }
                _ => {
                    self.local_dec.push_str(";\n");
                }
            }
            self.locals.insert(varname.into());
        }
    }
    fn ensure_exists_auto(&mut self, varname: &str) {
        if !self.locals.contains(varname) {
            self.local_dec.push_str(&format!("\tauto {varname};\n"));
            self.locals.insert(varname.into());
        }
    }
    fn put_bb(&mut self, code: IString, beg_idx: usize) {
        self.code.push_str(&format!("\tbb_{beg_idx}:\n"));
        self.code.push_str(&code);
    }
    fn new(
        args: &[VariableType],
        fn_name: &str,
        class_name: &str,
        ret_val: VariableType,
        is_virtual: bool,
    ) -> Self {
        let locals = HashSet::new();
        let mut local_dec = String::new();
        if is_virtual {
            local_dec.push_str(&format!("\tstd::shared_ptr<{class_name}> loc0a = std::static_pointer_cast<{class_name}>(this->shared_from_this());\n",class_name = class_name));
        }
        let mut sig = format!("{ret} {class_name}::{fn_name}(", ret = ret_val.c_type());
        let mut arg_iter = args.iter();
        let mut arg_index = 0;
        if is_virtual {
            arg_index += 1;
        }
        match arg_iter.next() {
            Some(arg) => {
                let ctype = arg.c_type();
                let postifx = arg.type_postifx();
                println!("{ctype} loc{arg_index}{postifx}");
                sig.push_str(&format!("{ctype} loc{arg_index}{postifx}"));
                arg_index += 1;
            }
            None => (),
        }
        for arg in arg_iter {
            let ctype = arg.c_type();
            let postifx = arg.type_postifx();
            sig.push_str(&format!(",{ctype} loc{arg_index}{postifx}"));
            arg_index += 1;
        }
        sig.push(')');
        let includes = format!("#include \"{class_name}.hpp\"\n");
        Self {
            signature: sig.into(),
            class_name: class_name.into(),
            local_dec,
            locals,
            im_idx: 0,
            code: String::new(),
            fn_name: fn_name.into(),
            includes,
        }
    }
    fn get_im_name(&mut self) -> IString {
        let im_name = format!("i{}", self.im_idx);
        self.im_idx += 1;
        im_name.into()
    }
    fn final_code(self) -> IString {
        format!(
            "{includes}{signature}{{\n{local_dec}{code}}}",
            includes = self.includes,
            signature = self.signature,
            local_dec = self.local_dec,
            code = self.code
        )
        .into()
    }
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
impl CompilationContext {
    fn write_stdlib(target_path: &PathBuf) -> std::io::Result<()> {
        //java_cs_lang_cs_Object;
        let mut object_out = target_path.clone();
        object_out.push("java_cs_lang_cs_Object");
        object_out.set_extension("hpp");
        if !object_out.exists() {
            let mut object_out = std::fs::File::create(object_out)?;
            object_out.write_all(java_cs_lang_cs_Object)?;
        }
        let mut runtime_out = target_path.clone();
        runtime_out.push("runtime");
        runtime_out.set_extension("hpp");
        if !runtime_out.exists() {
            let mut runtime_out = std::fs::File::create(runtime_out)?;
            runtime_out.write_all(runtime)?;
        }
        let mut java_cs_lang_cs_String_out = target_path.clone();
        java_cs_lang_cs_String_out.push("java_cs_lang_cs_String");
        java_cs_lang_cs_String_out.set_extension("hpp");
        if !java_cs_lang_cs_String_out.exists() {
            let mut java_cs_lang_cs_String_out = std::fs::File::create(java_cs_lang_cs_String_out)?;
            java_cs_lang_cs_String_out.write_all(java_cs_lang_cs_String)?;
        }

        //include_stdlib_header_file!(runtime);
        Ok(())
    }
    fn new(ca: &ConvertionArgs) -> Result<Self, BytecodeImportError> {
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
            classes.push(Class::from_java_class(&class));
        }
        println!("\r Finished stage 2(Conversion) of JVM bytecode to C++ translation.");
        std::fs::create_dir_all(&ca.out).unwrap();
        Self::write_stdlib(&ca.out).unwrap();
        for (index, class) in classes.iter().enumerate() {
            print_progress(index, classes.len());
            let mut path = ca.out.clone();
            path.push(class.name());
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
            for (sname, smethod) in class.static_methods() {
                let mut path = ca.out.clone();
                path.push(&format!("{}_{}", class.name(), sname));
                path.set_extension("cpp");
                let mut cout = std::fs::File::create(path)?;
                cpp_codegen::create_method_impl(&mut cout, smethod);
            }
            for (sname, smethod) in class.virtual_methods() {
                let mut path = ca.out.clone();
                path.push(&format!("{}_{}", class.name(), sname));
                path.set_extension("cpp");
                let mut cout = std::fs::File::create(path)?;
                cpp_codegen::create_method_impl(&mut cout, smethod);
            }
        }
        println!(
            "\r Finished stage 4(Generating Source files) of JVM bytecode to C++ translation."
        );
        todo!();
    }
}
fn main() {
    let args = ConvertionArgs::parse();
    println!("args:{args:?}");
    CompilationContext::new(&args).unwrap();
}

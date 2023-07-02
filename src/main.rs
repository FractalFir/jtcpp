mod importer;
mod fatops;
mod basic_block;
use crate::fatops::FatOp;
use std::io::Write;
use crate::importer::{ImportedJavaClass,BytecodeImportError};
use clap::Parser;
use std::path::PathBuf;
pub type IString = Box<str>;
use basic_block::BasicBlock;
fn method_name_to_c_name(method_name:&str)->IString{
    match method_name{
        "<init>"=>"_init_".into(),
        _=>method_name.into()
    }
}
fn class_path_to_class_mangled(class_path:&str)->IString{
    let mut out = String::with_capacity(class_path.len());
    let mut sequences = class_path.split('/');
    match sequences.next(){
        Some(prefix)=>out.push_str(prefix),
        None=>(),
    }
    for seq in sequences{
        out.push_str("_cs_");
        out.push_str(seq)
    }
    let out = out.replace('$',"_dolsig_");
    out.into()
}
fn desc_to_mangled(desc:&str)->IString{
    let mut classname_beg = 0;
    let mut within_class = false;
    let mut res = String::new();
    for (index,curr) in desc.chars().enumerate(){
        if curr == 'L'{
            within_class = true;
            classname_beg = index + 1;
        }
        if curr == ';'{
            within_class = false;
            let class = &desc[classname_beg..index];
            let class = class_path_to_class_mangled(class);
            res.push_str(&class);
            res.push_str("_as_");
            continue;
        }
        if curr == '('{
            res.push_str("_ab_");
            continue;
        }
        else if curr == ')'{
            res.push_str("ae_");
            continue;
        }
        if !within_class{
            res.push(curr);
        }
    }
    res.replace('[',"_arr_").into()
}
fn mangle_method_name(class: &str, method: &str, desc: &str) -> IString {
    let class = class_path_to_class_mangled(class);
    let desc = desc_to_mangled(desc);
    let method = method_name_to_c_name(method);
    format!("{class}_ce_{method}_ne_{desc}").into_boxed_str()
}
fn mangle_method_name_partial(method: &str, desc: &str) -> IString {
    let desc = desc_to_mangled(desc);
    let method = method_name_to_c_name(method);
    format!("{method}_meth_{desc}").into()
}
#[derive(Debug,Clone)]
enum VariableType{
    Void,
    Char,
    Bool,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ObjectRef{name: IString},
    ArrayRef(Box<VariableType>),
}
impl VariableType{
    fn c_type(&self)->IString{
        match self{
            Self::Float=>"float".into(),
            Self::Double=>"double".into(),
            Self::Long=>"long".into(),
            Self::Int=>"int".into(),
            Self::Bool=>"bool".into(),
            Self::Byte=>"char".into(),
            Self::Short=>"short".into(),
            Self::Char=>"short".into(),
            Self::Void=>"void".into(),
            Self::ObjectRef{name}=>name.clone(),
            Self::ArrayRef(atype)=>format!("{}[]",atype.c_type()).into(),
            //_=>todo!("Can't get ctype of {self:?}!"),
        }
    }
    fn type_postifx(&self)->IString{
        match self{
            Self::Float=>"f".into(),
            Self::Double=>"d".into(),
            Self::Long=>"l".into(),
            Self::Int=>"i".into(),
            Self::Bool=>"z".into(),
            Self::Byte=>"b".into(),
            Self::Short=>"s".into(),
            Self::ObjectRef{name}=>"a".into(),
            Self::ArrayRef(atype)=>format!("arr_{}",atype.c_type()).into(),
            _=>todo!("Can't get type postifx of {self:?}!"),
        }
    }
}
pub(crate) fn field_desc_str_to_ftype(desc_str: &str, th: usize) -> VariableType {
    let beg = desc_str.chars().nth(th).unwrap() ;
    match beg{
        'B' => VariableType::Byte,
        'C' => VariableType::Char,
        'D' => VariableType::Double,
        'F' => VariableType::Float,
        'I' => VariableType::Int,
        'J' => VariableType::Long,
        'L' => VariableType::ObjectRef { name: class_path_to_class_mangled(desc_str[(th+1)..(desc_str.len() - 1)].split(';').next().unwrap().into()) },
        '[' => VariableType::ArrayRef(Box::new(field_desc_str_to_ftype(desc_str,th+1))),
        'S' => VariableType::Short,
        'Z' => VariableType::Bool,
        'V' => VariableType::Void,
        _ => panic!("Invalid field descriptor!\"{desc_str}\". beg:{beg}"),
    }
}
pub(crate) fn field_descriptor_to_ftype(descriptor: u16, class: &ImportedJavaClass) -> VariableType {
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
struct Method{
    name:IString,
    ops:Box<[FatOp]>,
    args:Vec<VariableType>,
    ret_val:VariableType,
}
fn method_desc_to_args(desc:&str)->(Vec<VariableType>,VariableType){
    let arg_beg = desc.chars().position(|c| c == '(').unwrap() + 1;
    let arg_end = desc.chars().position(|c| c == ')').unwrap();
    let mut arg_desc = &desc[arg_beg..arg_end];
    let ret_val = field_desc_str_to_ftype(desc,arg_end + 1);
    let mut within_class = false;
    let mut args = Vec::new();
    for (index,curr) in arg_desc.chars().enumerate(){
        if !within_class{
            args.push(field_desc_str_to_ftype(arg_desc,index));
        }
        if curr == 'L'{
            within_class = true;
        }
        if curr == ';'{
            within_class = false;
        }
    }
    (args,ret_val)
}
impl Method{ 
    pub(crate) fn from_raw_method(method:&crate::importer::Method,name:&str,jc:&ImportedJavaClass)->Method{
        let name:IString = name.into();
        let (args,ret_val) = method_desc_to_args(method.descriptor(&jc));
        let ops = match method.bytecode(){
            Some(ops)=>fatops::expand_ops(ops,jc),
            None=>[].into(),
        };
        Method{name,args,ret_val,ops}
    }
    fn into_bbs(&self)->Vec<(usize,BasicBlock)>{
        let mut jump_targets = Vec::with_capacity(self.ops.len());
        for op in self.ops.iter(){
            match op.jump_target(){
                Some(targets)=>{
                    targets.iter().for_each(|target|{jump_targets.push(*target)});
                },
                None=>(),
            }
        }
        //println!("jump_targets:{jump_targets:?}");
        jump_targets.sort();
        jump_targets.dedup();
        let mut bbs = Vec::new();
        let mut bb_beg = 0;
        for (index,op) in self.ops.iter().enumerate(){
            println!("{index}:{op:?}");
            if jump_targets.contains(&index){
                bbs.push((bb_beg,BasicBlock::new(&self.ops[bb_beg..index],bb_beg)));
                bb_beg = index;
            }
        }
        if bb_beg < self.ops.len(){
            bbs.push((bb_beg,BasicBlock::new(&self.ops[bb_beg..],bb_beg)));
        }
        bbs.into()
    }
    fn link_bbs(bbs:&mut [(usize,BasicBlock)]) {
        //TODO:Link em
    }
    fn codegen(&self)->IString{
        let mut bbs = self.into_bbs();
        Self::link_bbs(&mut bbs);
        println!("bbs:{bbs:?}");
        let mut cg = MethodCG::new(&self.args,&self.name,self.ret_val.clone());
        for basic_block in bbs{
            basic_block.1.codegen(&mut cg);
        }
        cg.final_code()
    }
}

use std::collections::{HashSet,HashMap};
struct MethodCG{
    includes:String,
    fn_name:IString,
    signature:IString,
    local_dec:String,
    locals:HashSet<IString>,
    im_idx:usize,
    code:String,
}
impl MethodCG{
    fn add_include(&mut self,file:&str){
        //TODO:Handle double includes!
        self.includes.push_str("#include \"");
        self.includes.push_str(file);
        self.includes.push_str(".h\";\n");
    }
    fn ensure_exists(&mut self,varname:&str,vartype:&VariableType){
        if !self.locals.contains(varname){
            let ctype = vartype.c_type();
            self.local_dec.push_str(&format!("\t{ctype} {varname};\n"));
            self.locals.insert(varname.into());
        }
    }
    fn put_bb(&mut self,code:IString,beg_idx:usize){
        self.code.push_str(&format!("\tbb_{beg_idx}:\n"));
        self.code.push_str(&code);
    }
    fn new(args:&[VariableType],fn_name:&str,ret_val:VariableType)->Self{
        let mut sig = format!("{} {fn_name}(",ret_val.c_type());
        let mut arg_iter = args.iter().enumerate();
        match arg_iter.next(){
            Some((arg_index,arg))=>{
                let ctype = arg.c_type();
                let postifx = arg.type_postifx();
                sig.push_str(&format!("{ctype} loc{arg_index}{postifx}"));
            }
            None=>(),
        }
        for (arg_index,arg) in arg_iter{
            let ctype = arg.c_type();
            let postifx = arg.type_postifx();
            sig.push_str(&format!(",{ctype} loc{arg_index}{postifx}"));
        }
        sig.push(')');
        Self{signature:sig.into(),local_dec:String::new(),locals:HashSet::new(),im_idx:0,code:String::new(),fn_name:fn_name.into(),includes:String::new()}
    }
    fn get_im_name(&mut self)->IString{
        let im_name = format!("i{}",self.im_idx);
        self.im_idx += 1;
        im_name.into()
    }
    fn final_code(self)->IString{
        format!("{includes}{signature}{{\n{local_dec}{code}}}",includes = self.includes,signature = self.signature,local_dec = self.local_dec,code = self.code).into()
    }
}
#[test]
fn cg_sqr_mag(){
    let mut m_cg = MethodCG::new(&[VariableType::Float,VariableType::Float],"sqr",VariableType::Float);
    let bb = BasicBlock::new(&[
        FatOp::FLoad(0),FatOp::FLoad(0),FatOp::FMul,FatOp::FLoad(1),FatOp::FLoad(1),FatOp::FMul,FatOp::FAdd,FatOp::FReturn,
    ],0);
    bb.codegen(&mut m_cg);
    let final_code = m_cg.final_code();
}  
#[test]
fn cg_factorial(){
    let method = Method{name:"bt".into(),ops:Box::new([
            FatOp::IConst(1),FatOp::IStore(1),FatOp::IConst(2),FatOp::IStore(2),
            FatOp::ILoad(2),FatOp::ILoad(0),FatOp::IfICmpGreater(13),FatOp::ILoad(1),
            FatOp::ILoad(2),FatOp::IMul,FatOp::IStore(1),FatOp::IInc(2,1),
            FatOp::GoTo(4),FatOp::ILoad(1),FatOp::IReturn,
        ]),args:vec![VariableType::Int],ret_val:VariableType::Int
    };
    let code = method.codegen();
    panic!("code:{code}");
}  
#[test]
fn cg_divide(){
    let method = Method{name:"bt".into(),ops:Box::new([
            FatOp::ILoad(0),FatOp::ILoad(1),FatOp::IDiv,FatOp::IReturn,
        ]),args:vec![VariableType::Int,VariableType::Int],ret_val:VariableType::Int
    };
    let code = method.codegen();
    panic!("code:{code}");
}  
#[derive(Debug,Parser)]
#[command(author, version, about, long_about = None)]
struct ConvertionArgs{
    // Source files to load and convert to C.
    #[arg(short,long)]
    source_files:Vec<PathBuf>,
    // Target directory
    #[arg(short, long)]
    out:PathBuf,
}
struct CompilationContext{
    classes:HashMap<IString,Class>,
    methods:HashMap<IString,Method>,
}
struct Class{
    name:IString,
    parrent:IString,
    fields:Vec<(IString,VariableType)>,
    static_fields:Vec<(IString,VariableType)>,
    static_methods:Vec<(IString,Method)>,
}
impl Class{
    fn from_java_class(java_class:&importer::ImportedJavaClass)->Self{
        let name = match java_class.lookup_class(java_class.this_class()){
            Some(name)=>name,
            None=>{
                eprintln!("\nMalformed class file! `this_class` field did not refer to a valid class!");
                std::process::exit(ERR_THIS_INVALID);
            },
        };
        let class_name = class_path_to_class_mangled(name);
        let parrent = match java_class.lookup_class(java_class.super_class()){
            Some(parrent)=>parrent,
            None=>{
                eprintln!("\nMalformed class file! `super_class` field did not refer to a valid class!");
                std::process::exit(ERR_SUPER_INVALID);
            },
        };
        let parrent = class_path_to_class_mangled(parrent);
        let mut fields:Vec<(IString,VariableType)> = Vec::with_capacity(java_class.fields().len());
        let mut static_fields:Vec<(IString,VariableType)> = Vec::with_capacity(java_class.fields().len());
        for field in java_class.fields(){
            let (name_index, descriptor_index) = (field.name_index, field.descriptor_index);
            let name = java_class.lookup_utf8(name_index).unwrap();
            let ftype = field_descriptor_to_ftype(descriptor_index, java_class);
            if field.flags.is_static() {
                static_fields.push((name.into(), ftype));
            } else {
                fields.push((name.into(), ftype));
            }
        }
        let mut static_methods:Vec<(IString,Method)> = Vec::with_capacity(java_class.methods().len());
        let mut virtual_methods:Vec<(IString,Method)> = Vec::with_capacity(java_class.methods().len());
        for method in java_class.methods(){
            if method.is_virtual(java_class) {
                let mangled_name = method.mangled_name(java_class);
                let mut method = Method::from_raw_method(&method,&mangled_name,&java_class);
                method.args.insert(0,VariableType::ObjectRef { name: class_name.clone()});
                virtual_methods.push((mangled_name,method));
            }
            else{
                let mangled_name = method.mangled_name(java_class);
                let method = Method::from_raw_method(&method,&mangled_name,&java_class);
                static_methods.push((mangled_name,method));
               
            }
        }
        Class{name:class_name,parrent,fields,static_fields,static_methods}
        //todo!("name:{name} parrent:{parrent} fields:{fields:?} static_fields:{static_fields:?}");
    }
    fn write_header<W:Write>(&self,hout:&mut W)->std::io::Result<()>{
        let mut header_includes = format!("#pragma once\n#include <stdbool.h>\n #include \"{super_class}.h\"\n",super_class=self.parrent);
        let mut header_fields = format!("//Fielddef BEGINS\n\n#define {class_name}_FIELDS {super_class}_FIELDS \\\n",class_name=self.name,super_class=self.parrent);
        for (fname,field_type) in &self.fields{
            header_fields.push_str(&format!("\t{ctype} {fname}; \\\n",ctype=field_type.c_type()));
        }
        header_fields.push_str("//Fielddef ENDS\n\n");
        let mut header_class = &format!("typedef struct {class_name} {{\n\t{class_name}_FIELDS \n}} {class_name};\n",class_name=self.name);
        //let mut method_defs = format!("");
        let final_header = format!("{header_includes} {header_fields} {header_class}");
        hout.write_all(&final_header.into_bytes())
    }
}
const ERR_NO_EXT:i32 = 1;
const ERR_BAD_EXT:i32 = 2;
const ERR_FOPEN_FAIL:i32 = 3;
const ERR_THIS_INVALID:i32 = 4;
const ERR_SUPER_INVALID:i32 = 5;
const ERR_BAD_OUT:i32 = 6;
const ERR_HEADER_IO_FAIL:i32 = 7;
const PROGRESS_BAR_SIZE:usize = 50;
fn print_progress(curr:usize,whole:usize){
    print!("\r{curr}/{whole} \t");
    let fract = ((curr as f64 / whole as f64)*(PROGRESS_BAR_SIZE as f64)).round() as usize;
    for i in 0..PROGRESS_BAR_SIZE{
        if i < fract{
            print!("█");
        }
        else{
            print!("░");
        }
    }
    std::io::stdout().flush();
}
impl CompilationContext{
    fn new(ca:&ConvertionArgs)->Result<Self,BytecodeImportError>{
        let mut loaded_classes = Vec::new();
        for (index,path) in ca.source_files.iter().enumerate(){
            let path_disp = path.display();
            let extension = path.extension();
            print_progress(index,ca.source_files.len());
            let extension = match extension{
                Some(extension)=>extension,
                None=>{
                    eprintln!("\nFile at {path_disp} has no extension, so it can't be determied if it is either .class or .jar, and can't be compiled!");
                    std::process::exit(ERR_NO_EXT);
                }
            };
            match extension.to_str(){
                Some("jar")=>{
                    let mut src = match std::fs::File::open(path){
                        Ok(src)=>src,
                        Err(err)=>{
                            eprintln!("\nFile at {path_disp} can't be opened because {err:?}!");
                            std::process::exit(ERR_FOPEN_FAIL);
                        }
                    };
                    let classes = importer::load_jar(&mut src)?;
                    loaded_classes.extend(classes);
                },
                Some("class")=>{
                    let mut src = match std::fs::File::open(path){
                        Ok(src)=>src,
                        Err(err)=>{
                            eprintln!("\nFile at {path_disp} can't be opened because {err:?}!");
                            std::process::exit(ERR_FOPEN_FAIL);
                        }
                    };
                    let class = importer::load_class(&mut src)?;
                    loaded_classes.push(class);
                },
                _=>{
                    eprintln!("\nfile at {path_disp} is neither .class nor .jar, and can't be compiled!");
                    std::process::exit(ERR_BAD_EXT);
                },
            };
            println!("\rSuccessfully loaded file {path_disp}!                           ");
        }
        println!("\r Finished stage 1(Import) of JVM bytecode to C translation.");
        let mut classes = Vec::with_capacity(loaded_classes.len());
        for (index,class) in loaded_classes.iter().enumerate(){
            print_progress(index,loaded_classes.len());
            classes.push(Class::from_java_class(&class));
        }
        println!("\r Finished stage 2(Conversion) of JVM bytecode to C translation.");
        std::fs::create_dir_all(&ca.out);
        for (index,class) in classes.iter().enumerate(){
            print_progress(index,classes.len());
            let mut path = ca.out.clone();
            path.push(&*class.name);
            path.set_extension("h");
            let hout = std::fs::File::create(&path);
            let mut hout = match hout{
                Ok(hout)=>hout,
                Err(err)=>{
                    eprintln!("\nCan't create file at {path}!",path=path.display());
                    std::process::exit(ERR_BAD_OUT);
                },
            };
            match class.write_header(&mut hout){
                Ok(_)=>(),
                Err(err)=>{
                    eprintln!("\nCan't write header at path{path}, beacuse {err:?}!",path=path.display());
                    std::process::exit(ERR_HEADER_IO_FAIL);
                },
            }
            println!("\rcreating file at path:{}                                        ",path.display());
        }
        println!("\r Finished stage 3(Generating headers) of JVM bytecode to C translation.");
        for (index,class) in classes.iter().enumerate(){
            print_progress(index,classes.len());
            for (sname,smethod) in &class.static_methods{
                let mut path = ca.out.clone();
                path.push(&**sname);
                path.set_extension("c");
                let mut cout = std::fs::File::create(path)?;
                let mut code = smethod.codegen();
                cout.write_all(&code.into_boxed_bytes())?;
           }
        }
        todo!();
    }   
}
fn main(){
    let args = ConvertionArgs::parse();
    println!("args:{args:?}");
    CompilationContext::new(&args).unwrap();
    
}
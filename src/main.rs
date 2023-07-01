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
fn class_path_to_class_mangled(class_path:&str)->IString{
    let mut out = String::with_capacity(class_path.len());
    let mut sequences = class_path.split('/');
    match sequences.next(){
        Some(prefix)=>out.push_str(prefix),
        None=>(),
    }
    for seq in sequences{
        out.push_str("_csep_");
        out.push_str(seq)
    }
    out.into()
}
fn desc_to_mangled(desc:&str)->IString{
    desc.replace('(',"_abeg_").replace(')',"_aend_").replace(';', "_onameend_").into()
}
fn mangle_method_name(class: &str, method: &str, desc: &str) -> IString {
    let class = class_path_to_class_mangled(class);
    let desc = desc_to_mangled(desc);
    format!("{class}_meth_{method}{desc}").into_boxed_str()
}
fn mangle_method_name_partial(method: &str, desc: &str) -> IString {
    let desc = desc_to_mangled(desc);
    format!("{method}_meth_{desc}").into()
}
#[derive(Debug,Clone)]
enum VariableType{
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
            _=>todo!("Can't get ctype of {self:?}!"),
        }
    }
    fn type_postifx(&self)->IString{
        match self{
            Self::Float=>"f".into(),
            Self::Double=>"d".into(),
            Self::Long=>"l".into(),
            Self::Int=>"i".into(),
            _=>todo!("Can't get type postifx of {self:?}!"),
        }
    }
}
pub(crate) fn field_desc_str_to_ftype(desc_str: &str, th: usize) -> VariableType {
    match desc_str.chars().nth(th).unwrap() {
        'B' => VariableType::Byte,
        'C' => VariableType::Char,
        'D' => VariableType::Double,
        'F' => VariableType::Float,
        'I' => VariableType::Int,
        'J' => VariableType::Long,
        'L' => VariableType::ObjectRef { name: desc_str[(th+1)..(desc_str.len() - 1)].into() },
        '[' => VariableType::ArrayRef(Box::new(field_desc_str_to_ftype(desc_str,th+1))),
        'S' => VariableType::Short,
        'Z' => VariableType::Bool,
        _ => panic!("Invalid field descriptor!\"{desc_str}\""),
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
impl Method{
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
        println!("jump_targets:{jump_targets:?}");
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
    fn_name:IString,
    signature:IString,
    local_dec:String,
    locals:HashSet<IString>,
    im_idx:usize,
    code:String,
}
impl MethodCG{
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
        Self{signature:sig.into(),local_dec:String::new(),locals:HashSet::new(),im_idx:0,code:String::new(),fn_name:fn_name.into()}
    }
    fn get_im_name(&mut self)->IString{
        let im_name = format!("i{}",self.im_idx);
        self.im_idx += 1;
        im_name.into()
    }
    fn final_code(self)->IString{
        format!("{}{{\n{}{}}}",self.signature,self.local_dec,self.code).into()
    }
}
struct FatClass{
    name:IString,
    super_name:IString,
    fields:Vec<VariableType>,
    methods:Vec<Method>,
    vmethods:Vec<Method>,
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
    parrent:IString,
}
impl Class{
    fn from_java_class(java_class:&importer::ImportedJavaClass)->Self{
        todo!();
    }
    fn write_header<W:Write>(out:&mut W)->std::io::Result<()>{
        todo!();
    }
}
const ERR_NO_EXT:i32 = 1;
const ERR_BAD_EXT:i32 = 2;
const ERR_FOPEN_FAIL:i32 = 3;
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
            println!("\rSuccessfully loaded file {path_disp}!                           ");
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
        }
        println!("\r Finished stage 1(Import) of JVM bytecode to C translation.");
        todo!();
    }   
}
fn main(){
    let args = ConvertionArgs::parse();
    println!("args:{args:?}");
    CompilationContext::new(&args).unwrap();
    
}
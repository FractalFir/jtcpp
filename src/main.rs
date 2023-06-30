mod importer;
mod fatops;
use crate::fatops::FatOp;
use crate::importer::ImportedJavaClass;
pub type IString = Box<str>;
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
enum FieldType{
    Char,
    Bool,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ObjectRef{name: IString},
    ArrayRef(Box<FieldType>),
}
impl FieldType{
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
pub(crate) fn field_desc_str_to_ftype(desc_str: &str, th: usize) -> FieldType {
    match desc_str.chars().nth(th).unwrap() {
        'B' => FieldType::Byte,
        'C' => FieldType::Char,
        'D' => FieldType::Double,
        'F' => FieldType::Float,
        'I' => FieldType::Int,
        'J' => FieldType::Long,
        'L' => FieldType::ObjectRef { name: desc_str[(th+1)..(desc_str.len() - 1)].into() },
        '[' => FieldType::ArrayRef(Box::new(field_desc_str_to_ftype(desc_str,th+1))),
        'S' => FieldType::Short,
        'Z' => FieldType::Bool,
        _ => panic!("Invalid field descriptor!\"{desc_str}\""),
    }
}
pub(crate) fn field_descriptor_to_ftype(descriptor: u16, class: &ImportedJavaClass) -> FieldType {
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
    args:Vec<FieldType>,
    ret_val:FieldType,
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
                bbs.push((bb_beg,BasicBlock{input:Vec::new(),output:Vec::new(),beg_idx:bb_beg,ops:&self.ops[bb_beg..index]}));
                bb_beg = index;
            }
        }
        if bb_beg < self.ops.len(){
            bbs.push((bb_beg,BasicBlock{input:Vec::new(),output:Vec::new(),beg_idx:bb_beg,ops:&self.ops[bb_beg..]}));
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
#[derive(Debug)]
struct BasicBlock<'a>{
    input:Vec<FieldType>,
    output:Vec<FieldType>,
    ops:&'a [FatOp],
    beg_idx:usize,
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
    fn ensure_exists(&mut self,varname:&str,vartype:&FieldType){
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
    fn new(args:&[FieldType],fn_name:&str,ret_val:FieldType)->Self{
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
macro_rules! basic_op_impl{
    ($vstack:ident,$cg:ident,$code:ident,$otype:literal,$op:literal) => {
        {
            let b = $vstack.pop().unwrap();
            let a = $vstack.pop().unwrap();
            let im_name = $cg.get_im_name();
            $code.push_str(&format!(concat!("\t",$otype," {} = {} ",$op," {};\n"),im_name,a,b));
            $vstack.push(im_name);
        }
    };
}
impl<'a> BasicBlock<'a>{
    fn vstack(&self)->Vec<IString>{
        Vec::new()
    }
    fn codegen(&self,cg:&mut MethodCG){
        let mut vstack = self.vstack();
        let mut code = String::new();
        for op in self.ops.iter(){
            match op{
                FatOp::FLoad(index)=>{
                    vstack.push(format!("loc{index}f").into());
                }
                FatOp::FMul=>basic_op_impl!(vstack,cg,code,"float","*"),
                FatOp::FAdd=>basic_op_impl!(vstack,cg,code,"float","+"),
                FatOp::FDiv=>basic_op_impl!(vstack,cg,code,"float","/"),
                FatOp::FSub=>basic_op_impl!(vstack,cg,code,"float","-"),
                FatOp::DMul=>basic_op_impl!(vstack,cg,code,"dobule","*"),
                FatOp::DAdd=>basic_op_impl!(vstack,cg,code,"dobule","+"),
                FatOp::DDiv=>basic_op_impl!(vstack,cg,code,"dobule","/"),
                FatOp::DSub=>basic_op_impl!(vstack,cg,code,"dobule","-"),
                FatOp::IMul=>basic_op_impl!(vstack,cg,code,"int","*"),
                FatOp::IAdd=>basic_op_impl!(vstack,cg,code,"int","+"),
                FatOp::IDiv=>basic_op_impl!(vstack,cg,code,"int","/"),
                FatOp::IRem=>basic_op_impl!(vstack,cg,code,"int","%"),
                FatOp::ISub=>basic_op_impl!(vstack,cg,code,"int","-"),
                FatOp::LMul=>basic_op_impl!(vstack,cg,code,"long","*"),
                FatOp::LAdd=>basic_op_impl!(vstack,cg,code,"long","+"),
                FatOp::LDiv=>basic_op_impl!(vstack,cg,code,"long","/"),
                FatOp::LSub=>basic_op_impl!(vstack,cg,code,"long","-"),
                FatOp::FConst(float)=>{
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tfloat {im_name} = {float};\n"));
                    vstack.push(im_name);
                },
                FatOp::IConst(int)=>{
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tint {im_name} = {int};\n"));
                    vstack.push(im_name);
                },
                FatOp::ILoad(var_idx)=>{
                    vstack.push(format!("loc{var_idx}i").into_boxed_str());
                },
                FatOp::IStore(var_idx)=>{
                    let vname = format!("loc{var_idx}i").into_boxed_str();
                    cg.ensure_exists(&vname,&FieldType::Int);
                    let set = vstack.pop().unwrap();
                    code.push_str(&format!("\t{vname} = {set};\n"));
                },
                FatOp::FReturn | FatOp::DReturn | FatOp::IReturn =>{
                    let ret = vstack.pop().unwrap();
                    code.push_str(&format!("\treturn {ret};\n"));
                }
                FatOp::IfNotZero(jump_pos)=>{
                    let val = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({val} != 0)goto bb_{jump_pos};\n"));
                }
                FatOp::IfIGreterEqual(jump_pos)=>{
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({a} >= {b})goto bb_{jump_pos};\n"));
                }
                FatOp::IfICmpGreater(jump_pos)=>{
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({a} > {b})goto bb_{jump_pos};\n"));
                }
                FatOp::GoTo(jump_pos)=>{
                    code.push_str(&format!("\tgoto bb_{jump_pos};\n"));
                },
                FatOp::IInc(variable,increment)=>{
                    let vname = format!("loc{variable}i").into_boxed_str();
                    cg.ensure_exists(&vname,&FieldType::Int);
                    code.push_str(&format!("\t{vname} = {vname} + {increment};\n"));
                }
                _=>todo!("Can't convert {op:?} to C."),
            }
        }
        println!("code:{code:?}");
        cg.put_bb(code.into(),self.beg_idx);
    }
}
struct FatClass{
    name:IString,
    super_name:IString,
    fields:Vec<FieldType>,
    methods:Vec<Method>,
    vmethods:Vec<Method>,
}
#[test]
fn cg_sqr_mag(){
    let mut m_cg = MethodCG::new(&[FieldType::Float,FieldType::Float],"sqr",FieldType::Float);
    let bb = BasicBlock{input:Vec::new(),output:Vec::new(),ops:&[
        FatOp::FLoad(0),FatOp::FLoad(0),FatOp::FMul,FatOp::FLoad(1),FatOp::FLoad(1),FatOp::FMul,FatOp::FAdd,FatOp::FReturn,
    ],beg_idx:0};
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
        ]),args:vec![FieldType::Int],ret_val:FieldType::Int
    };
    let code = method.codegen();
    panic!("code:{code}");
}  
#[test]
fn cg_divide(){
    let method = Method{name:"bt".into(),ops:Box::new([
            FatOp::ILoad(0),FatOp::ILoad(1),FatOp::IDiv,FatOp::IReturn,
        ]),args:vec![FieldType::Int,FieldType::Int],ret_val:FieldType::Int
    };
    let code = method.codegen();
    panic!("code:{code}");
}  
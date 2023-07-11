use super::IncludeBuilder;
use crate::fatops::FatOp;
use crate::IString;
use std::collections::HashSet;
use std::io::Write;
use crate::VariableType;
struct MethodWriter {
    includes: super::IncludeBuilder,
    code: String,
    sig: IString,
    ident_level: usize,
    local_decl: String,
    vstack: Vec<(VariableType,IString)>,
    locals: HashSet<IString>,
    im_id:usize,
}
enum LocalKind {
    ObjectRef,
    Float,
}
impl MethodWriter {
    pub(crate) fn esnsure_local_exists(&mut self, id: u8, kind: LocalKind,ctype:&str) -> IString {
        let local = self.get_local(id, kind);
        if !self.locals.contains(&local){
            self.local_decl.push_str(&format!("\t{ctype} {local};\n"));
            self.locals.insert(local.clone());
        }
        local
    }
    pub(crate) fn get_local(&self, id: u8, kind: LocalKind) -> IString {
        match kind {
            LocalKind::ObjectRef => format!("l{id}a"),
            LocalKind::Float => format!("l{id}f"),
        }
        .into()
    }
    pub(crate) fn use_debuginfo(&self) -> bool {
        cfg!(debug_assertions)
    }
    pub(crate) fn new() -> Self {
        Self {
            vstack: Vec::with_capacity(64),
            code: String::new(),
            sig: "".into(),
            includes: IncludeBuilder::new(""),
            ident_level: 1,
            local_decl: String::new(),
            locals: HashSet::new(),
            im_id:0,
        }
    }
    pub(crate) fn begin_bb(&mut self, index: usize) {
        self.write_ident();
        self.code.push_str(&format!("bb_{index}:\n"));
    }
    pub(crate) fn begin_scope(&mut self) {
        self.write_ident();
        self.ident_level += 1;
        self.code.push_str("{\n");
    }
    pub(crate) fn end_scope(&mut self) {
        self.ident_level -= 1;
        self.write_ident();
        self.code.push_str("}\n");
    }
    fn write_ident(&mut self) {
        for _ in 0..self.ident_level {
            self.code.push('\t');
        }
    }
    pub(crate) fn add_include(&mut self, include: &str) {
        self.includes.add_include(include);
    }
    pub(crate) fn set_sig(&mut self, sig: &str) {
        self.sig = sig.into();
    }
    pub(crate) fn write_op(&mut self, curr_op: &FatOp, code: &str) {
        if self.use_debuginfo() {
            self.write_ident();
            self.code.push_str(&format!("//{curr_op:?}\n"));
        }
        self.write_ident();
        self.code.push_str(code);
        self.code.push('\n');
    }
    pub(crate) fn vstack_push(&mut self, vvar: &str,vtype:VariableType) {
        self.vstack.push((vtype,vvar.into()))
    }
    pub(crate) fn get_intermidiate(&mut self)->IString{
        let im = format!("i{id}",id = self.im_id);
        self.im_id += 1;
        im.into()
    }
    pub(crate) fn push_locals(&mut self,local:&str,decl:&str){
        if !self.locals.contains(local){
            self.locals.insert(local.into());
            self.local_decl.push_str(decl);
        }
    }
    pub(crate) fn vstack_pop(&mut self)->Option<(VariableType,IString)>{self.vstack.pop()}
    pub(crate) fn final_code(&self) -> IString {
        format!(
            "{includes}{sig}{{\n{local_decl}{code}}}",
            includes = self.includes.get_code(),
            code = self.code,
            sig = self.sig,
            local_decl = self.local_decl
        )
        .into()
    }
}
enum BasicBlock {
    Raw { ops: Box<[FatOp]>, starts: usize },
    //Scope(Box<[BasicBlock]>),
}
macro_rules! load_impl{
    ($mw:ident,$index:ident,$kind:expr,$vtype:expr)=>{{
            let local = $mw.get_local(*$index, $kind);
            $mw.vstack_push(&local,$vtype);
            "".into()
        }
    }
}
macro_rules! store_impl{
    ($mw:ident,$index:ident,$kind:expr)=>{{
            let (vtype,value):(VariableType,IString) = $mw.vstack_pop().unwrap();
            let local:IString = $mw.esnsure_local_exists(*$index, $kind,&vtype.c_type());
            format!("{local} = {value};")
        }
    }
}
macro_rules! get_field_impl {
    ($mw:ident,$field_name:ident,$vartype:expr) => {{
        let field_owner = $mw.vstack_pop().unwrap();
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name,$vartype);
        let field_owner = field_owner.1;
        format!(
            "\t{ctype} {im_name} = {field_owner}->{field_name};\n",field_name = $field_name, ctype = $vartype.c_type()
        )
        }
    };
}
macro_rules! get_static_impl {
    ($mw:ident,$field_owner:ident,$static_name:ident,$vartype:expr) => {{
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name,$vartype);
        format!(
            "\t{ctype} {im_name} = {field_owner}::{static_name};\n",static_name = $static_name, ctype = $vartype.c_type(),field_owner = $field_owner
        )
        }
    };
}
macro_rules! arthm_impl {
    ($mw:ident,$vartype:expr,$op:literal) => {{
        let (atype,a) = $mw.vstack_pop().unwrap();
        let (btype,b) = $mw.vstack_pop().unwrap();
        assert_eq!(atype,btype);
        assert_eq!(atype,$vartype);
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name,$vartype);
        format!(concat!("{ctype} {im} = {a}",$op,"{b};\n"),ctype = $vartype.c_type(),im = im_name,a = a, b = b)
    }};
}
macro_rules! convert_impl {
    ($mw:ident,$src_type:expr,$target:expr) => {{
        let (src,val) = $mw.vstack_pop().unwrap();
        assert_eq!(src,$src_type);
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name,$target);
        format!("{target} {im_name} = ({target}){val}",target = $target.c_type())
    }};
}
fn write_op(op: &FatOp, mw: &mut MethodWriter) {
    let code = match op {
        FatOp::ALoad(index) => load_impl!(mw,index,LocalKind::ObjectRef,VariableType::ObjectRef{name: "Unknown".into()}),
        FatOp::FLoad(index) => load_impl!(mw,index,LocalKind::Float,VariableType::Float),
        FatOp::FConst(value)=>{
            //let constant = &format("{value}");
            mw.vstack_push(&format!("{value:.16}f"),VariableType::Float);
            "".into()
        }
        FatOp::FGetField(_class_name, field_name) => get_field_impl!(mw,field_name,VariableType::Float),
        FatOp::FGetStatic(class_name, field_name) => get_static_impl!(mw,class_name,field_name,VariableType::Float),
        FatOp::AStore(index) => store_impl!(mw,index,LocalKind::ObjectRef),
        FatOp::FStore(index) => store_impl!(mw,index,LocalKind::Float),
        FatOp::FPutField(_class_name, field_name) => {
            let float_value = mw.vstack_pop().unwrap();
            let field_owner =  mw.vstack_pop().unwrap();
            assert_eq!(float_value.0,VariableType::Float);
            let field_owner = field_owner.1;
            let float_value = float_value.1;
            format!("\t{field_owner}->{field_name} = {float_value};\n")
        }
        FatOp::FAdd => arthm_impl!(mw,VariableType::Float,"+"),
        FatOp::FSub => arthm_impl!(mw,VariableType::Float,"-"),
        FatOp::FMul => arthm_impl!(mw,VariableType::Float,"*"),
        FatOp::FDiv => arthm_impl!(mw,VariableType::Float,"/"),
        FatOp::F2D => convert_impl!(mw,VariableType::Float,VariableType::Double),
        FatOp::D2F => convert_impl!(mw,VariableType::Double,VariableType::Float),
        FatOp::FReturn | FatOp::AReturn =>{
            let value = mw.vstack_pop().unwrap().1;
            format!("return {value};")
        }
        FatOp::Return => "return;".into(),
        FatOp::Dup =>{
            let value = mw.vstack_pop().unwrap();
            mw.vstack_push(&value.1,value.0.clone());
            mw.vstack_push(&value.1,value.0.clone());
            "".into()
        }
        FatOp::New(name)=>{
            let im = mw.get_intermidiate();
            mw.vstack_push(&im,VariableType::ObjectRef { name:name.clone() });
            format!("{name}* {im} = new {name}();")
        }
        FatOp::InvokeVirtual(_class_name, vmethod_name, args, ret) => {
            let mut code = String::new();
            let argc = args.len();
            let mut args: Vec<IString> = Vec::with_capacity(argc);
            for _ in 0..(argc + 1){
                args.push(mw.vstack_pop().unwrap().1);
            }
            args.reverse();
            let mut args = args.iter();
            let objref = args.next().unwrap();
            if *ret == crate::VariableType::Void {
                code.push_str(&format!("{objref}->{vmethod_name}("));
            } else {
                let im_name = mw.get_intermidiate();
                code.push_str(&format!(
                    "{ret} {im_name} = {objref}->{vmethod_name}(",
                    ret = ret.c_type()
                ));
                mw.vstack_push(&im_name,ret.clone());
            }
            match args.next() {
                Some(arg) => code.push_str(arg),
                None => (),
            }
            for arg in args {
                code.push(',');
                code.push_str(arg);
            }
            code.push_str(");");
            code
        }
        FatOp::InvokeStatic(method_class_name, method_name, args, ret) => {
            mw.add_include(method_class_name);
            let mut code = String::new();
            let argc = args.len();
            let mut args: Vec<IString> = Vec::with_capacity(argc);
            for _ in 0..argc{
                args.push(mw.vstack_pop().unwrap().1);
            }
            args.reverse();
            let mut args = args.iter();
            if *ret == crate::VariableType::Void {
                code.push_str(&format!("{method_class_name}::{method_name}("));
            } else {
                let im_name = mw.get_intermidiate();
                code.push_str(&format!(
                    "{ret} {im_name} = {method_class_name}::{method_name}(",
                    ret = ret.c_type()
                ));
                mw.vstack_push(&im_name,ret.clone());
            }
            match args.next() {
                Some(arg) => code.push_str(arg),
                None => (),
            }
            for arg in args {
                code.push(',');
                code.push_str(arg);
            }
            code.push_str(");");
            code
        }
        _ => todo!("Unsuported op:\"{op:?}\""),
    };
    mw.write_op(op, &code);
}
impl BasicBlock {
    fn starts(&self) -> usize {
        match self {
            Self::Raw { starts, .. } => *starts,
        }
    }
    fn write(&self, writer: &mut MethodWriter) {
        writer.begin_bb(self.starts());
        writer.begin_scope();
        match self {
            Self::Raw { ops, .. } => {
                for op in ops.iter() {
                    write_op(op, writer);
                }
            }
        }
        writer.end_scope();
    }
}
//May be unneded?
fn bb_unroll(basic_spans: &[(usize, &[FatOp])]) -> Box<[BasicBlock]> {
    // Iteretes all the spans, returning only those which have at least one jump, which goes forward.
    let forward_spans = basic_spans.iter().filter(|(start, ops)| {
        ops.iter()
            .map(|op| op.jump_target())
            .any(|targets| targets.is_some_and(|targets| targets.iter().any(|pos| pos > start)))
    });
    todo!();
}
fn fat_ops_to_bb_tree(fatops: &[FatOp]) -> Box<[BasicBlock]> {
    let mut jump_targets = Vec::with_capacity(fatops.len() / 3);
    for op in fatops {
        if let Some(targets) = op.jump_target() {
            targets.iter().for_each(|target| jump_targets.push(*target));
        }
    }
    let mut basic_spans: Vec<(usize, &[FatOp])> = Vec::new();
    let mut bb_beg = 0;
    for (index, _op) in fatops.iter().enumerate() {
        //println!("{index}:{op:?}");
        if jump_targets.contains(&index) {
            basic_spans.push((bb_beg, &fatops[bb_beg..index]));
            bb_beg = index;
        }
    }
    if bb_beg < fatops.len() {
        basic_spans.push((bb_beg, &fatops[bb_beg..]));
    }
    // Detect which BBs jump forward, and which are jumpe overm to create spans!
    // may be unneded? -> bb_unroll(&basic_spans)
    basic_spans
        .iter()
        .map(|(starts, ops)| BasicBlock::Raw {
            ops: (*ops).into(),
            starts: *starts,
        })
        .collect()
}
fn push_method_sig_args(target: &mut String, method_name: &str, method: &crate::Method) {
    let mut curr_id = if method.is_virtual(){1}else{0};
    target.push_str(&format!(
        "{ret} {method_name}(",
        ret = method.ret_val().c_type()
    ));
    let mut margs = method.args().iter();
    //println!("\n\t{name}::{method_name}->{margs:?}",name = self.name);
    match margs.next() {
        Some(arg) => {
            target.push_str(&format!("{ctype} l{curr_id}{postfix}",ctype = &arg.c_type(),postfix = arg.type_postifx()));
            curr_id += 1;
        },
        None => (),
    }
    for arg in margs {
        target.push_str(&format!(",{ctype} l{curr_id}{postfix}",ctype = &arg.c_type(),postfix = arg.type_postifx()));
        curr_id += 1;
    }
    target.push_str(")");
}
pub(crate) fn create_method_impl(
    mut out: impl Write,
    method: &crate::Method,
) -> Result<(), std::io::Error> {
    let bb_tree = fat_ops_to_bb_tree(method.ops());
    let mut writer = MethodWriter::new();
    let mut fn_sig = String::new();
    push_method_sig_args(
        &mut fn_sig,
        &format!(
            "{class_name}::{method_name}",
            class_name = method.class_name(),
            method_name = method.name()
        ),
        method,
    );
    writer.set_sig(&fn_sig);
    writer.add_include(method.class_name());
    if method.is_virtual(){
        writer.push_locals("loc0a", &format!("\t{class}* l0a = this;\n",class = method.class_name()));
    }
    for bb in bb_tree.iter() {
        bb.write(&mut writer);
    }
    out.write_all(writer.final_code().as_bytes())?;
    Ok(())
}

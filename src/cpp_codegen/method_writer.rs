use super::IncludeBuilder;
use crate::{fatops::FatOp, IString, VariableType};
use std::collections::HashSet;
pub(crate) enum LocalKind {
    ObjectRef,
    Float,
    Double,
    Int,
    Long,
}
pub(crate) struct MethodWriter {
    includes: super::IncludeBuilder,
    code: String,
    sig: IString,
    ident_level: usize,
    local_decl: String,
    vstack: Vec<(VariableType, IString)>,
    locals: HashSet<IString>,
    local_types:Vec<Option<VariableType>>,
    im_id: usize,
}
impl MethodWriter {
    pub(crate) fn ensure_local_exists(&mut self, id: u8, kind: LocalKind, vtype: VariableType) -> (IString,VariableType)  {
        let local = self.get_local(id, kind);
        if !self.locals.contains(&local.0) {
            self.local_decl.push_str(&format!("\t{ctype} {local_name};\n",local_name = local.0,ctype = vtype.c_type()));
            self.set_local_type(id,vtype);
            self.locals.insert(local.0.clone());
        }
        local
    }
    fn set_local_type(&mut self, id:u8,vtype: VariableType){
        while self.local_types.len() <= id as usize{
            self.local_types.push(None);
        }
        self.local_types[id as usize] = Some(vtype);
    }
    pub(crate) fn get_local_type(&self,id:u8)->VariableType{
        //println!("local_types: {:?}, local_id:{id}, local_type:{:?}",self.local_types,self.local_types.get(id as usize));
        match self.local_types.get(id as usize).cloned().flatten()
        {
            Some(vt)=>vt,
            None=>VariableType::unknown(),
        }
    }
    pub(crate) fn get_local(&self, id: u8, kind: LocalKind) -> (IString,VariableType) {
        match kind {
            LocalKind::ObjectRef => (format!("l{id}a").into(),self.get_local_type(id)),
            LocalKind::Float => (format!("l{id}f").into(),VariableType::Float),
            LocalKind::Double => (format!("l{id}d").into(),VariableType::Double),
            LocalKind::Int => (format!("l{id}i").into(),VariableType::Int),
            LocalKind::Long => (format!("l{id}l").into(),VariableType::Long),
        }
    }
    pub(crate) fn use_debuginfo(&self) -> bool {
        cfg!(debug_assertions)
    }
    pub(crate) fn new(args:&[VariableType]) -> Self {
        Self {
            vstack: Vec::with_capacity(64),
            code: String::new(),
            sig: "".into(),
            includes: IncludeBuilder::new(""),
            local_types: args.iter().map(|v|Some(v.clone())).collect(),
            ident_level: 1,
            local_decl: String::new(),
            locals: HashSet::new(),
            im_id: 0,
        }
    }
    pub(crate) fn begin_bb(&mut self, index: usize) {
        self.write_ident();
        self.code.push_str(&format!("bb{index}:\n"));
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
    pub(crate) fn write_raw(&mut self, code: &str){
        self.write_ident();
        self.code.push_str(code);
        self.code.push('\n');
    }
    pub(crate) fn write_op(&mut self, curr_op: &FatOp, code: &str) {
        if self.use_debuginfo() {
            self.write_ident();
            self.code.push_str(&format!("//{curr_op:?}\n"));
        }
        if code != ""{
            self.write_raw(code);
        }
    }
    pub(crate) fn vstack_push(&mut self, vvar: &str, vtype: VariableType) {
        self.vstack.push((vtype, vvar.into()))
    }
    pub(crate) fn get_intermidiate(&mut self) -> IString {
        let im = format!("i{id}", id = self.im_id);
        self.im_id += 1;
        im.into()
    }
    pub(crate) fn push_locals(&mut self, local: &str, decl: &str) {
        if !self.locals.contains(local) {
            self.locals.insert(local.into());
            self.local_decl.push_str(decl);
        }
    }
    #[allow(dead_code)]
    pub(crate) fn print_stack(&self) {
        println!("vstack:{:?}", self.vstack);
    }
    pub(crate) fn vstack_pop(&mut self) -> Option<(VariableType, IString)> {
        self.vstack.pop()
    }
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
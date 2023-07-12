use super::IncludeBuilder;
use crate::fatops::FatOp;
use crate::IString;
use crate::VariableType;
use std::collections::HashSet;
use std::io::Write;
struct MethodWriter {
    includes: super::IncludeBuilder,
    code: String,
    sig: IString,
    ident_level: usize,
    local_decl: String,
    vstack: Vec<(VariableType, IString)>,
    locals: HashSet<IString>,
    im_id: usize,
}
enum LocalKind {
    ObjectRef,
    Float,
    Int,
}
impl MethodWriter {
    pub(crate) fn esnsure_local_exists(&mut self, id: u8, kind: LocalKind, ctype: &str) -> IString {
        let local = self.get_local(id, kind);
        if !self.locals.contains(&local) {
            self.local_decl.push_str(&format!("\t{ctype} {local};\n"));
            self.locals.insert(local.clone());
        }
        local
    }
    pub(crate) fn get_local(&self, id: u8, kind: LocalKind) -> IString {
        match kind {
            LocalKind::ObjectRef => format!("l{id}a"),
            LocalKind::Float => format!("l{id}f"),
            LocalKind::Int => format!("l{id}i"),
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
    pub(crate) fn write_op(&mut self, curr_op: &FatOp, code: &str) {
        if self.use_debuginfo() {
            self.write_ident();
            self.code.push_str(&format!("//{curr_op:?}\n"));
        }
        self.write_ident();
        self.code.push_str(code);
        self.code.push('\n');
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
enum BasicBlock {
    Raw { ops: Box<[FatOp]>, starts: usize },
    //Scope(Box<[BasicBlock]>),
}
macro_rules! load_impl {
    ($mw:ident,$index:ident,$kind:expr,$vtype:expr) => {{
        let local = $mw.get_local(*$index, $kind);
        $mw.vstack_push(&local, $vtype);
        "".into()
    }};
}
macro_rules! store_impl {
    ($mw:ident,$index:ident,$kind:expr) => {{
        let (vtype, value): (VariableType, IString) = $mw.vstack_pop().unwrap();
        let local: IString = $mw.esnsure_local_exists(*$index, $kind, &vtype.c_type());
        format!("{local} = {value};")
    }};
}
macro_rules! get_field_impl {
    ($mw:ident,$field_name:ident,$vartype:expr) => {{
        let field_owner = $mw.vstack_pop().unwrap();
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name, $vartype);
        let field_owner = field_owner.1;
        format!(
            "{ctype} {im_name} = {field_owner}->{field_name};",
            field_name = $field_name,
            ctype = $vartype.c_type()
        )
    }};
}
macro_rules! set_field_impl {
    ($mw:ident,$field_name:ident,$vartype:expr) => {{
        let (valtype, value) = $mw.vstack_pop().unwrap();
        let field_owner = $mw.vstack_pop().unwrap();
        assert_eq!(valtype, $vartype);
        let field_owner = field_owner.1;
        format!(
            "{field_owner}->{field_name} = {value};",
            field_name = $field_name,
        )
    }};
    ($mw:ident,$field_name:ident) => {{
        let (_valtype, value) = $mw.vstack_pop().unwrap();
        let field_owner = $mw.vstack_pop().unwrap();
        //assert_eq!(valtype,$vartype);
        let field_owner = field_owner.1;
        format!(
            "{field_owner}->{field_name} = {value};",
            field_name = $field_name,
        )
    }};
}
macro_rules! get_static_impl {
    ($mw:ident,$field_owner:ident,$static_name:ident,$vartype:expr) => {{
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name, $vartype);
        if let Some(dep) = ($vartype).dependency() {
            $mw.add_include(dep);
        }
        $mw.add_include($field_owner);
        format!(
            "{ctype} {im_name} = {field_owner}::{static_name};",
            static_name = $static_name,
            ctype = $vartype.c_type(),
            field_owner = $field_owner
        )
    }};
}
macro_rules! set_static_impl {
    ($mw:ident,$field_owner:ident,$static_name:ident,$vartype:expr) => {{
        let (vtype, value) = $mw.vstack_pop().unwrap();
        assert_eq!(vtype, $vartype);
        format!(
            "{field_owner}::{static_name} = {value};",
            static_name = $static_name,
            field_owner = $field_owner
        )
    }};
}
macro_rules! arthm_impl {
    ($mw:ident,$vartype:expr,$op:literal) => {{
        let (btype, b) = $mw.vstack_pop().unwrap();
        let (atype, a) = $mw.vstack_pop().unwrap();
        assert_eq!(atype, btype);
        assert_eq!(atype, $vartype);
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name, $vartype);
        format!(
            concat!("{ctype} {im} = {a}", $op, "{b};"),
            ctype = $vartype.c_type(),
            im = im_name,
            a = a,
            b = b
        )
    }};
}
macro_rules! convert_impl {
    ($mw:ident,$src_type:expr,$target:expr) => {{
        let (src, val) = $mw.vstack_pop().unwrap();
        assert_eq!(src, $src_type);
        let im_name = $mw.get_intermidiate();
        $mw.vstack_push(&im_name, $target);
        format!(
            "{target} {im_name} = ({target}){val};",
            target = $target.c_type()
        )
    }};
}
macro_rules! conditional_impl {
    ($mw:ident,$cmp:literal,$target:ident) => {{
        let (_btype, b) = $mw.vstack_pop().unwrap();
        let (_atype, a) = $mw.vstack_pop().unwrap();
        //TODO: Consider chaecking types!
        format!(
            concat!("if({a}", $cmp, "{b}) goto bb{target};"),
            a = a,
            b = b,
            target = $target
        )
    }};
}
fn write_op(op: &FatOp, mw: &mut MethodWriter) {
    let code = match op {
        FatOp::ALoad(index) => load_impl!(
            mw,
            index,
            LocalKind::ObjectRef,
            VariableType::ObjectRef {
                name: "Unknown".into()
            }
        ),
        FatOp::FLoad(index) => load_impl!(mw, index, LocalKind::Float, VariableType::Float),
        FatOp::ILoad(index) => load_impl!(mw, index, LocalKind::Int, VariableType::Int),
        FatOp::FConst(value) => {
            //let constant = &format("{value}");
            mw.vstack_push(&format!("{value:.16}f"), VariableType::Float);
            "".into()
        }
        FatOp::IConst(value) => {
            //let constant = &format("{value}");
            mw.vstack_push(&format!("{value:}"), VariableType::Int);
            "".into()
        }
        FatOp::SConst(value) => {
            //let constant = &format("{value}");
            mw.vstack_push(&format!("{value:}"), VariableType::Int);
            "".into()
        }
        FatOp::AGetField {
            class_name: _,
            field_name,
            type_name,
        } => get_field_impl!(
            mw,
            field_name,
            VariableType::ObjectRef {
                name: type_name.clone()
            }
        ),
        FatOp::FGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Float)
        }
        FatOp::AGetStatic {
            class_name,
            static_name,
            type_name,
        } => get_static_impl!(
            mw,
            class_name,
            static_name,
            VariableType::ObjectRef {
                name: type_name.clone()
            }
        ),
        FatOp::AAGetField {
            class_name: _,
            field_name,
            atype,
        } => get_field_impl!(
            mw,
            field_name,
            VariableType::ArrayRef(Box::new(atype.clone()))
        ), //TODO: fix vstack type issues, readd ``
        FatOp::FGetStatic(class_name, static_name) => {
            get_static_impl!(mw, class_name, static_name, VariableType::Float)
        }
        FatOp::AAStore => {
            let (_value_type, value) = mw.vstack_pop().unwrap();
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array());
            assert_eq!(index_type, VariableType::Int);
            format!("{arr_ref}->Set({index},{value});")
        }
        FatOp::AALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(
                &im_name,
                VariableType::ObjectRef {
                    name: "unknown".into(),
                },
            );
            format!("auto {im_name} = {arr_ref}->Get({index});")
        }
        FatOp::ArrayLength => {
            let (_arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Int);
            format!("int {im_name} = {arr_ref}->GetLength();")
        }
        FatOp::AStore(index) => store_impl!(mw, index, LocalKind::ObjectRef),
        FatOp::FStore(index) => store_impl!(mw, index, LocalKind::Float),
        FatOp::IStore(index) => store_impl!(mw, index, LocalKind::Int),
        FatOp::APutField {
            class_name: _,
            field_name,
            type_name,
        } => set_field_impl!(
            mw,
            field_name,
            VariableType::ObjectRef {
                name: type_name.clone()
            }
        ),
        FatOp::AAPutField {
            class_name: _,
            field_name,
            atype: _,
        } => set_field_impl!(mw, field_name), //TODO: fix vstack type issues, readd `VariableType::ArrayRef(Box::new(atype.clone()))`
        FatOp::FPutField(_class_name, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Float)
        }
        FatOp::APutStatic {
            class_name,
            field_name,
            type_name,
        } => set_static_impl!(
            mw,
            field_name,
            class_name,
            VariableType::ObjectRef {
                name: type_name.clone()
            }
        ),
        FatOp::FPutStatic(class_name, field_name) => {
            set_static_impl!(mw, class_name, field_name, VariableType::Float)
        }
        FatOp::FAdd => arthm_impl!(mw, VariableType::Float, "+"),
        FatOp::FSub => arthm_impl!(mw, VariableType::Float, "-"),
        FatOp::FMul => arthm_impl!(mw, VariableType::Float, "*"),
        FatOp::FDiv => arthm_impl!(mw, VariableType::Float, "/"),
        FatOp::IInc(local, by) => {
            let local = mw.get_local(*local, LocalKind::Int);
            format!("{local} += {by};")
        }
        FatOp::FRem => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            assert_eq!(atype, btype);
            assert_eq!(atype, VariableType::Float);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Float);
            format!("float {im_name} = fmod({a},{b});")
        }
        FatOp::F2D => convert_impl!(mw, VariableType::Float, VariableType::Double),
        FatOp::D2F => convert_impl!(mw, VariableType::Double, VariableType::Float),
        FatOp::FReturn | FatOp::AReturn => {
            let value = mw.vstack_pop().unwrap().1;
            format!("return {value};")
        }
        FatOp::Return => "return;".into(),
        FatOp::IfIGreterEqual(target) => conditional_impl!(mw, ">=", target),
        FatOp::IfICmpNe(target) => conditional_impl!(mw, "!=", target),
        FatOp::GoTo(target) => format!("goto bb{target};"),
        FatOp::Dup => {
            let value = mw.vstack_pop().unwrap();
            mw.vstack_push(&value.1, value.0.clone());
            mw.vstack_push(&value.1, value.0.clone());
            "".into()
        }
        FatOp::Pop => {
            let _ = mw.vstack_pop().unwrap();
            "".into()
        }
        FatOp::New(name) => {
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::ObjectRef { name: name.clone() });
            format!("{name}* {im} = new {name}();")
        }
        FatOp::ANewArray(name) => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ObjectRef { name: name.clone() });
            format!("RuntimeArray<{name}*>* {im} = new RuntimeArray<{name}*>({length});")
        }
        FatOp::StringConst(const_string) => {
            let im_name = mw.get_intermidiate();
            mw.vstack_push(
                &im_name,
                VariableType::ObjectRef {
                    name: "java_cs_lang_cs_String".into(),
                },
            );
            format!("java_cs_lang_cs_String* {im_name} = new java_cs_lang_cs_String(u\"{const_string}\");")
        }
        FatOp::InvokeVirtual(_class_name, vmethod_name, args, ret) => {
            let mut code = String::new();
            let argc = args.len();
            let mut args: Vec<IString> = Vec::with_capacity(argc);
            for _ in 0..(argc + 1) {
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
                mw.vstack_push(&im_name, ret.clone());
            }
            if let Some(arg) = args.next() {
                code.push_str(arg)
            };
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
            for _ in 0..argc {
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
                mw.vstack_push(&im_name, ret.clone());
            }
            if let Some(arg) = args.next() {
                code.push_str(arg)
            };
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
    let _forward_spans = basic_spans.iter().filter(|(start, ops)| {
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
    let mut curr_id = if method.is_virtual() { 1 } else { 0 };
    target.push_str(&format!(
        "{ret} {method_name}(",
        ret = method.ret_val().c_type()
    ));
    let mut margs = method.args().iter();
    //println!("\n\t{name}::{method_name}->{margs:?}",name = self.name);
    match margs.next() {
        Some(arg) => {
            target.push_str(&format!(
                "{ctype} l{curr_id}{postfix}",
                ctype = &arg.c_type(),
                postfix = arg.type_postifx()
            ));
            curr_id += 1;
        }
        None => (),
    }
    for arg in margs {
        target.push_str(&format!(
            ",{ctype} l{curr_id}{postfix}",
            ctype = &arg.c_type(),
            postfix = arg.type_postifx()
        ));
        curr_id += 1;
    }
    target.push(')');
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
    if method.is_virtual() {
        writer.push_locals(
            "loc0a",
            &format!("\t{class}* l0a = this;\n", class = method.class_name()),
        );
    }
    for bb in bb_tree.iter() {
        bb.write(&mut writer);
    }
    out.write_all(writer.final_code().as_bytes())?;
    Ok(())
}

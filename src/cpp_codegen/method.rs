use super::IncludeBuilder;
use crate::{fatops::FatOp, ClassInfo, IString, VariableType};
use std::{collections::HashSet, io::Write};
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
    Double,
    Int,
    Long,
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
            LocalKind::Double => format!("l{id}d"),
            LocalKind::Int => format!("l{id}i"),
            LocalKind::Long => format!("l{id}l"),
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
    pub(self) fn print_stack(&self) {
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
        if !valtype.is_unknown() {
            //assert_eq!(valtype, $vartype);
        }
        let field_owner = field_owner.1;
        format!(
            "{field_owner}->{field_name} = {value};",
            field_name = $field_name,
        )
    }};
    ($mw:ident,$field_name:ident,$vartype:expr,$ctype:literal) => {{
        let (valtype, value) = $mw.vstack_pop().unwrap();
        let field_owner = $mw.vstack_pop().unwrap();
        assert!($vartype.assignable(&valtype));
        let field_owner = field_owner.1;
        format!(
            "{field_owner}->{field_name} = ({ctype}){value};",
            field_name = $field_name,
            ctype = $ctype
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
            $mw.add_include(&dep);
        }
        $mw.add_include(&*$field_owner.class_path());
        format!(
            "{ctype} {im_name} = {field_owner}::{static_name};",
            static_name = $static_name,
            ctype = $vartype.c_type(),
            field_owner = $field_owner.cpp_class()
        )
    }};
}
macro_rules! set_static_impl {
    ($mw:ident,$field_owner:ident,$static_name:ident,$vartype:expr) => {{
        let (vtype, value) = $mw.vstack_pop().unwrap();
        //debug_assert_eq!(vtype, $vartype);
        format!(
            "{field_owner}::{static_name} = {value};",
            static_name = $static_name,
            field_owner = $field_owner.cpp_class()
        )
    }};
}
macro_rules! arthm_impl {
    ($mw:ident,$vartype:expr,$op:literal) => {{
        let (btype, b) = $mw.vstack_pop().unwrap();
        let (atype, a) = $mw.vstack_pop().unwrap();
        assert!($vartype.assignable(&atype));
        assert!($vartype.assignable(&btype));
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
        //TODO: Consider checking types!
        format!(
            concat!("if({a}", $cmp, "{b}) goto bb{target};"),
            a = a,
            b = b,
            target = $target
        )
    }};
    ($mw:ident,$cmp:literal,$value:literal,$target:ident) => {{
        let (_atype, a) = $mw.vstack_pop().unwrap();
        //TODO: Consider checking types!
        format!(
            concat!("if({a}", $cmp, $value, ") goto bb{target};"),
            a = a,
            target = $target
        )
    }};
}
fn write_op(op: &FatOp, mw: &mut MethodWriter) {
    // Uncoment for more diagnosis
    //println!("");
    //mw.print_stack();
    //println!("op:{op:?}");
    let code = match op {
        FatOp::ALoad(index) => load_impl!(
            mw,
            index,
            LocalKind::ObjectRef,
            VariableType::ObjectRef(ClassInfo::unknown())
        ),
        FatOp::DLoad(index) => load_impl!(mw, index, LocalKind::Float, VariableType::Double),
        FatOp::FLoad(index) => load_impl!(mw, index, LocalKind::Float, VariableType::Float),
        FatOp::ILoad(index) => load_impl!(mw, index, LocalKind::Int, VariableType::Int),
        FatOp::LLoad(index) => load_impl!(mw, index, LocalKind::Int, VariableType::Long),
        FatOp::AConstNull => {
            mw.vstack_push("nullptr", VariableType::ObjectRef(ClassInfo::unknown()));
            "".into()
        }
        FatOp::FConst(value) => {
            mw.vstack_push(&format!("{value:.16}f"), VariableType::Float);
            "".into()
        }
        FatOp::DConst(value) => {
            mw.vstack_push(&format!("{value:.32}f"), VariableType::Double);
            "".into()
        }
        FatOp::BConst(value) => {
            mw.vstack_push(&format!("{value:}"), VariableType::Int);
            "".into()
        }
        FatOp::IConst(value) => {
            mw.vstack_push(&format!("{value:}"), VariableType::Int);
            "".into()
        }
        FatOp::LConst(value) => {
            mw.vstack_push(&format!("{value:}"), VariableType::Long);
            "".into()
        }
        FatOp::SConst(value) => {
            mw.vstack_push(&format!("{value:}"), VariableType::Int);
            "".into()
        }
        FatOp::AGetField {
            class_info: _,
            field_name,
            type_info,
        } => get_field_impl!(mw, field_name, VariableType::ObjectRef(type_info.clone())),
        FatOp::BGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Byte)
        }
        FatOp::CGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Char)
        }
        FatOp::DGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Double)
        }
        FatOp::FGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Float)
        }
        FatOp::IGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Int)
        }
        FatOp::LGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Long)
        }
        FatOp::SGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Short)
        }
        FatOp::ZGetField(_class_name, field_name) => {
            get_field_impl!(mw, field_name, VariableType::Bool)
        }

        FatOp::AGetStatic {
            class_info,
            static_name,
            type_info,
        } => get_static_impl!(
            mw,
            class_info,
            static_name,
            VariableType::ObjectRef(type_info.clone())
        ),
        FatOp::AAGetField {
            class_info: _,
            field_name,
            atype,
        } => get_field_impl!(
            mw,
            field_name,
            VariableType::ArrayRef(Box::new(atype.clone()))
        ),
        FatOp::BGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Byte)
        }
        FatOp::CGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Char)
        }
        FatOp::DGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Double)
        }
        FatOp::FGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Float)
        }
        FatOp::IGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Int)
        }
        FatOp::SGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Short)
        }
        FatOp::LGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Long)
        }
        FatOp::ZGetStatic(class_info, static_name) => {
            get_static_impl!(mw, class_info, static_name, VariableType::Bool)
        }
        FatOp::AAGetStatic {
            atype,
            class_info,
            static_name,
        } => {
            get_static_impl!(
                mw,
                class_info,
                static_name,
                VariableType::ArrayRef(Box::new(atype.clone()))
            )
        }
        FatOp::AAStore
        | FatOp::BAStore
        | FatOp::CAStore
        | FatOp::DAStore
        | FatOp::FAStore
        | FatOp::IAStore
        | FatOp::SAStore
        | FatOp::LAStore => {
            let (_value_type, value) = mw.vstack_pop().unwrap();
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            //assert!(arr_ref_type.is_array(),"arr_ref_type:{arr_ref_type:?}");
            assert_eq!(index_type, VariableType::Int);
            format!("{arr_ref}->Set({index},{value});")
        }
        FatOp::AALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::ObjectRef(ClassInfo::unknown()));
            format!("auto {im_name} = {arr_ref}->Get({index});")
        }
        FatOp::BALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Byte);
            format!("uint8_t {im_name} = (uint8_t){arr_ref}->Get({index});")
        }
        FatOp::CALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Char);
            format!("char16_t {im_name} = (char16_t){arr_ref}->Get({index});")
        }
        FatOp::DALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Double);
            format!("double {im_name} = (double){arr_ref}->Get({index});")
        }
        FatOp::FALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Float);
            format!("float {im_name} = (float){arr_ref}->Get({index});")
        }
        FatOp::IALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Int);
            format!("int {im_name} = (int){arr_ref}->Get({index});")
        }
        FatOp::LALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Long);
            format!("long {im_name} = (long){arr_ref}->Get({index});")
        }
        FatOp::SALoad => {
            let (index_type, index) = mw.vstack_pop().unwrap();
            let (arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            assert!(arr_ref_type.is_array() || arr_ref_type.is_unknown());
            assert_eq!(index_type, VariableType::Int);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Short);
            format!("short {im_name} = (short){arr_ref}->Get({index});")
        }
        FatOp::ArrayLength => {
            let (_arr_ref_type, arr_ref) = mw.vstack_pop().unwrap();
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Int);
            format!("int {im_name} = {arr_ref}->GetLength();")
        }
        FatOp::AStore(index) => store_impl!(mw, index, LocalKind::ObjectRef),
        FatOp::DStore(index) => store_impl!(mw, index, LocalKind::Double),
        FatOp::FStore(index) => store_impl!(mw, index, LocalKind::Float),
        FatOp::IStore(index) => store_impl!(mw, index, LocalKind::Int),
        FatOp::LStore(index) => store_impl!(mw, index, LocalKind::Long),
        FatOp::AAPutField {
            field_name,
            atype: _,
            ..
        } => set_field_impl!(mw, field_name), //TODO: fix vstack type issues, readd `VariableType::ArrayRef(Box::new(atype.clone()))`
        FatOp::APutField {
            class_info: _,
            field_name,
            type_info,
        } => set_field_impl!(mw, field_name, VariableType::ObjectRef(type_info.clone())),
        FatOp::BPutField(_, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Byte, "int8_t")
        }
        FatOp::CPutField(_, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Char, "char16_t")
        }
        FatOp::DPutField(_, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Double)
        }
        FatOp::FPutField(_class_name, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Float)
        }
        FatOp::IPutField(_class_name, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Int)
        }
        FatOp::SPutField(_class_name, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Short, "int16_t")
        }
        FatOp::LPutField(_class_name, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Long)
        }
        FatOp::ZPutField(_, field_name) => {
            set_field_impl!(mw, field_name, VariableType::Bool, "bool")
        }
        FatOp::APutStatic {
            class_info,
            field_name,
            type_info,
        } => set_static_impl!(
            mw,
            class_info,
            field_name,
            VariableType::ObjectRef(type_info.clone())
        ),
        FatOp::BPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Byte)
        }
        FatOp::CPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Char)
        }
        FatOp::DPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Double)
        }
        FatOp::FPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Float)
        }
        FatOp::IPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Int)
        }
        FatOp::LPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Long)
        }
        FatOp::SPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Short)
        }
        FatOp::ZPutStatic(class_info, field_name) => {
            set_static_impl!(mw, class_info, field_name, VariableType::Bool)
        }
        FatOp::AAPutStatic {
            class_info,
            field_name,
            atype,
        } => set_static_impl!(
            mw,
            class_info,
            field_name,
            VariableType::ArrayRef(Box::new(atype.clone()))
        ),

        FatOp::IInc(local, by) => {
            let local = mw.get_local(*local, LocalKind::Int);
            format!("{local} += {by};")
        }
        FatOp::DAdd => arthm_impl!(mw, VariableType::Double, "+"),
        FatOp::DSub => arthm_impl!(mw, VariableType::Double, "-"),
        FatOp::DMul => arthm_impl!(mw, VariableType::Double, "*"),
        FatOp::DDiv => arthm_impl!(mw, VariableType::Double, "/"),
        FatOp::DNeg => {
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Double);
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Double);
            format!("double {im} = !{a};")
        }
        FatOp::DRem => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            assert_eq!(atype, btype);
            assert_eq!(atype, VariableType::Float);
            let im_name = mw.get_intermidiate();
            mw.vstack_push(&im_name, VariableType::Double);
            format!("double {im_name} = fmod({a},{b});")
        }
        FatOp::FAdd => arthm_impl!(mw, VariableType::Float, "+"),
        FatOp::FSub => arthm_impl!(mw, VariableType::Float, "-"),
        FatOp::FMul => arthm_impl!(mw, VariableType::Float, "*"),
        FatOp::FDiv => arthm_impl!(mw, VariableType::Float, "/"),
        FatOp::FNeg => {
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Float);
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Float);
            format!("float {im} = !{a};")
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
        FatOp::IAdd => arthm_impl!(mw, VariableType::Int, "+"),
        FatOp::ISub => arthm_impl!(mw, VariableType::Int, "-"),
        FatOp::IMul => arthm_impl!(mw, VariableType::Int, "*"),
        FatOp::IDiv => arthm_impl!(mw, VariableType::Int, "/"),
        FatOp::IShl => arthm_impl!(mw, VariableType::Int, "<<"),
        FatOp::IShr => arthm_impl!(mw, VariableType::Int, ">>"),
        FatOp::IUShr => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Int);
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            format!("int {im} = (int)(((unsigned int){a})>>{b});")
        }
        FatOp::IAnd => arthm_impl!(mw, VariableType::Int, "&"),
        FatOp::IOr => arthm_impl!(mw, VariableType::Int, "|"),
        FatOp::IXOr => arthm_impl!(mw, VariableType::Int, "^"),
        FatOp::INeg => {
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Int);
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            format!("int {im} = !{a};")
        }
        FatOp::IRem => arthm_impl!(mw, VariableType::Long, "%"),
        FatOp::LAdd => arthm_impl!(mw, VariableType::Long, "+"),
        FatOp::LSub => arthm_impl!(mw, VariableType::Long, "-"),
        FatOp::LMul => arthm_impl!(mw, VariableType::Long, "*"),
        FatOp::LDiv => arthm_impl!(mw, VariableType::Long, "/"),
        FatOp::LShl => arthm_impl!(mw, VariableType::Long, "<<"),
        FatOp::LShr => arthm_impl!(mw, VariableType::Long, ">>"),
        FatOp::LUShr => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Long);
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Long);
            format!("long {im} = (long)(((unsigned long){a})>>{b});")
        }
        FatOp::LAnd => arthm_impl!(mw, VariableType::Long, "&"),
        FatOp::LOr => arthm_impl!(mw, VariableType::Long, "|"),
        FatOp::LXOr => arthm_impl!(mw, VariableType::Long, "^"),
        FatOp::LNeg => {
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Long);
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Long);
            format!("long {im} = !{a};")
        }
        FatOp::LRem => arthm_impl!(mw, VariableType::Long, "%"),
        FatOp::D2F => convert_impl!(mw, VariableType::Double, VariableType::Float),
        FatOp::D2I => convert_impl!(mw, VariableType::Double, VariableType::Int),
        FatOp::D2L => convert_impl!(mw, VariableType::Double, VariableType::Long),
        FatOp::F2D => convert_impl!(mw, VariableType::Float, VariableType::Double),
        FatOp::F2I => convert_impl!(mw, VariableType::Float, VariableType::Int),
        FatOp::I2B => convert_impl!(mw, VariableType::Int, VariableType::Byte),
        FatOp::I2C => convert_impl!(mw, VariableType::Int, VariableType::Char),
        FatOp::I2D => convert_impl!(mw, VariableType::Int, VariableType::Double),
        FatOp::I2F => convert_impl!(mw, VariableType::Int, VariableType::Float),
        FatOp::I2S => convert_impl!(mw, VariableType::Int, VariableType::Short),
        FatOp::F2L => convert_impl!(mw, VariableType::Float, VariableType::Long),
        FatOp::I2L => convert_impl!(mw, VariableType::Int, VariableType::Long),
        FatOp::L2I => convert_impl!(mw, VariableType::Long, VariableType::Int),
        FatOp::L2F => convert_impl!(mw, VariableType::Long, VariableType::Float),
        FatOp::L2D => convert_impl!(mw, VariableType::Long, VariableType::Double),
        FatOp::AReturn | FatOp::DReturn | FatOp::FReturn | FatOp::IReturn | FatOp::LReturn => {
            let value = mw.vstack_pop().unwrap().1;
            format!("return {value};")
        }
        FatOp::Return => "return;".into(),
        FatOp::DCmpL => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Double);
            debug_assert_eq!(atype, btype);
            // if A > B 1
            // if A == B 0
            // if A < B -1
            // if A | B == NaN, then -1
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            format!("int {im} = {a} > {b} ? 1: ({a} == {b}? 0: -1);")
        }
        FatOp::DCmpG => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Double);
            debug_assert_eq!(atype, btype);
            // if A > B 1
            // if A == B 0
            // if A < B -1
            // if A | B == NaN, then 1
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            format!("int {im} = {a} < {b}? -1 : ({a} == {b}? 0: 1);")
        }
        FatOp::FCmpL => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Float);
            debug_assert_eq!(atype, btype);
            // if A > B 1
            // if A == B 0
            // if A < B -1
            // if A | B == NaN, then -1
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            format!("int {im} = {a} > {b} ? 1: ({a} == {b}? 0: -1);")
        }
        FatOp::FCmpG => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Float);
            debug_assert_eq!(atype, btype);
            // if A > B 1
            // if A == B 0
            // if A < B -1
            // if A | B == NaN, then 1
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            format!("int {im} = {a} < {b}? -1 : ({a} == {b}? 0: 1);")
        }
        FatOp::LCmp => {
            let (btype, b) = mw.vstack_pop().unwrap();
            let (atype, a) = mw.vstack_pop().unwrap();
            debug_assert_eq!(atype, VariableType::Long);
            debug_assert_eq!(atype, btype);
            // if A > B 1
            // if A == B 0
            // if A < B -1
            // if A | B == NaN, then 1
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            format!("int {im} = {a} < {b}? -1 : ({a} == {b}? 0: 1);")
        }
        FatOp::IfIGreterEqual(target) => conditional_impl!(mw, ">=", target),
        FatOp::IfICmpNe(target) => conditional_impl!(mw, "!=", target),
        FatOp::IfICmpEq(target) => conditional_impl!(mw, "==", target),
        FatOp::IfICmpLess(target) => conditional_impl!(mw, "<", target),
        FatOp::IfICmpLessEqual(target) => conditional_impl!(mw, "<=", target),
        FatOp::IfICmpGreater(target) => conditional_impl!(mw, ">", target),
        FatOp::IfZero(target) => conditional_impl!(mw, "==", "0", target),
        FatOp::IfLessEqualZero(target) => conditional_impl!(mw, "<=", "0", target),
        FatOp::IfGreterEqualZero(target) => conditional_impl!(mw, ">=", "0", target),
        FatOp::IfGreterZero(target) => conditional_impl!(mw, ">", "0", target),
        FatOp::IfLessZero(target) => conditional_impl!(mw, "<", "0", target),
        FatOp::IfNotNull(target) => conditional_impl!(mw, "!=", "nullptr", target),
        FatOp::IfNull(target) => conditional_impl!(mw, "==", "nullptr", target),
        FatOp::IfACmpEq(target) => conditional_impl!(mw, "==", target),
        FatOp::IfACmpNe(target) => conditional_impl!(mw, "!=", target),
        FatOp::GoTo(target) => format!("goto bb{target};"),
        FatOp::Dup => {
            let value = mw.vstack_pop().unwrap();
            mw.vstack_push(&value.1, value.0.clone());
            mw.vstack_push(&value.1, value.0.clone());
            "".into()
        }
        FatOp::DupX1 => {
            let value = mw.vstack_pop().unwrap();
            let value_2 = mw.vstack_pop().unwrap();
            mw.vstack_push(&value.1, value.0.clone());
            mw.vstack_push(&value_2.1, value_2.0.clone());
            mw.vstack_push(&value.1, value.0.clone());
            "".into()
        }
        FatOp::DupX2 => {
            let value = mw.vstack_pop().unwrap();
            let value_2 = mw.vstack_pop().unwrap();
            if value_2.0.is_wide() {
                mw.vstack_push(&value.1, value.0.clone());
                mw.vstack_push(&value_2.1, value_2.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
            } else {
                let value_3 = mw.vstack_pop().unwrap();
                mw.vstack_push(&value.1, value.0.clone());
                mw.vstack_push(&value_3.1, value_3.0.clone());
                mw.vstack_push(&value_2.1, value_2.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
            }
            "".into()
        }
        FatOp::Swap => {
            let value = mw.vstack_pop().unwrap();
            let value_2 = mw.vstack_pop().unwrap();
            if value.0.is_wide() | value_2.0.is_wide() {
                panic!("Swap attempted at values bigger than word(Double or Long).");
            }
            mw.vstack_push(&value.1, value.0.clone());
            mw.vstack_push(&value_2.1, value_2.0.clone());
            "".into()
        }
        FatOp::Dup2X1 => {
            let value = mw.vstack_pop().unwrap();
            if value.0.is_wide() {
                let value_2 = mw.vstack_pop().unwrap();
                mw.vstack_push(&value.1, value.0.clone());
                mw.vstack_push(&value_2.1, value_2.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
            } else {
                let value_2 = mw.vstack_pop().unwrap();
                let value_3 = mw.vstack_pop().unwrap();
                mw.vstack_push(&value_2.1, value_2.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
                mw.vstack_push(&value_3.1, value_3.0.clone());
                mw.vstack_push(&value_2.1, value_2.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
            }
            "".into()
        }
        FatOp::Dup2 => {
            let value = mw.vstack_pop().unwrap();
            if value.0.is_wide() {
                mw.vstack_push(&value.1, value.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
            } else {
                let value_2 = mw.vstack_pop().unwrap();
                mw.vstack_push(&value_2.1, value_2.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
                mw.vstack_push(&value_2.1, value_2.0.clone());
                mw.vstack_push(&value.1, value.0.clone());
            }
            "".into()
        }
        FatOp::Pop2 => {
            let value = mw.vstack_pop().unwrap();
            if !value.0.is_wide() {
                let _ = mw.vstack_pop().unwrap();
            }
            "".into()
        }
        FatOp::Pop => {
            let _ = mw.vstack_pop().unwrap();
            "".into()
        }
        FatOp::New(class_info) => {
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::ObjectRef(class_info.clone()));
            mw.add_include(&*class_info.class_path());
            format!(
                "ManagedPointer<{name}> {im} = new_managed({name},);",
                name = class_info.cpp_class()
            )
        }
        FatOp::CheckedCast(class_info) => {
            let (vtype, value) = mw.vstack_pop().unwrap();
            mw.vstack_push(&value, vtype);
            mw.add_include(&*class_info.class_path());
            format!(
                "if(typeid(*{value}) != typeid({name}) = throw new java::lang::ClassCastException();",
                name = class_info.cpp_class()
            )
        }
        FatOp::InstanceOf(class_info) => {
            let (vtype, value) = mw.vstack_pop().unwrap();
            let im = mw.get_intermidiate();
            mw.vstack_push(&im, VariableType::Int);
            mw.add_include(&*class_info.class_path());
            format!(
                "int {im} = typeid(*{value}) != typeid({name}) ? 1:0;",
                name = class_info.cpp_class()
            )
        }
        FatOp::ANewArray(class_info) => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(
                &im,
                VariableType::ArrayRef(Box::new(VariableType::ObjectRef(class_info.clone()))),
            );
            mw.add_include(&*class_info.class_path());
            format!(
                "ManagedPointer<RuntimeArray<ManagedPointer<{name}>>> {im} = managed_from_raw(new RuntimeArray<ManagedPointer<{name}>>({length}));",
                name = class_info.cpp_class()
            )
        }
        FatOp::BNewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Byte)));
            format!("ManagedPointer<RuntimeArray<int8_t>> {im} = managed_from_raw(new RuntimeArray<int8_t>({length}));",)
        }
        FatOp::CNewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Char)));
            format!("ManagedPointer<RuntimeArray<char16_t>> {im} = managed_from_raw(new RuntimeArray<char16_t>({length}));",)
        }
        FatOp::DNewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Double)));
            format!("ManagedPointer<RuntimeArray<double>> {im} = managed_from_raw(new RuntimeArray<double>({length}));",)
        }
        FatOp::FNewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Float)));
            format!("ManagedPointer<RuntimeArray<float>> {im} = managed_from_raw(new RuntimeArray<float>({length}));",)
        }
        FatOp::INewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Int)));
            format!("ManagedPointer<RuntimeArray<int>> {im} = managed_from_raw(new RuntimeArray<int>({length}));",)
        }
        FatOp::LNewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Long)));
            format!("ManagedPointer<RuntimeArray<long>> {im} = managed_from_raw(new RuntimeArray<long>({length}));",)
        }
        FatOp::SNewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Short)));
            format!("ManagedPointer<RuntimeArray<short>> {im} = managed_from_raw(new RuntimeArray<short>({length}));",)
        }
        FatOp::ZNewArray => {
            let im = mw.get_intermidiate();
            let (length_type, length) = mw.vstack_pop().unwrap();
            assert_eq!(length_type, VariableType::Int);
            mw.vstack_push(&im, VariableType::ArrayRef(Box::new(VariableType::Bool)));
            format!("ManagedPointer<RuntimeArray<bool>> {im} = managed_from_raw(new RuntimeArray<bool>({length}));",)
        }
        FatOp::StringConst(const_string) => {
            let im_name = mw.get_intermidiate();
            mw.add_include("java_cs_lang_cs_String");
            mw.vstack_push(
                &im_name,
                VariableType::ObjectRef(crate::fatops::ClassInfo::from_java_path(
                    "java/lang/String",
                )),
            );
            format!("ManagedPointer<java::lang::String> {im_name} = managed_from_raw(new java::lang::String(u\"{const_string}\"));")
        }
        FatOp::ClassConst(class_info) => {
            let im_name = mw.get_intermidiate();
            mw.add_include("java_cs_lang_cs_Class");
            mw.add_include(class_info.cpp_class());
            mw.vstack_push(
                &im_name,
                VariableType::ObjectRef(crate::fatops::ClassInfo::from_java_path(
                    "java/lang/Class",
                )),
            );
            format!("ManagedPointer<java::lang::Class> {im_name} = managed_from_raw(new java::lang::Class(u\"{class_name}\"));",
            class_name = class_info.cpp_class())
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
            if let Some(dep) = ret.dependency() {
                mw.add_include(&dep);
            }
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
        FatOp::InvokeInterface(_class_name, vmethod_name, args, ret) => {
            let mut code = String::new();
            let argc = args.len();
            let mut args: Vec<IString> = Vec::with_capacity(argc);
            for _ in 0..(argc + 1) {
                args.push(mw.vstack_pop().unwrap().1);
            }
            args.reverse();
            let mut args = args.iter();
            let objref = args.next().unwrap();
            if let Some(dep) = ret.dependency() {
                mw.add_include(&dep);
            }
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
        FatOp::InvokeSpecial(method_class_info, method_name, args, ret) => {
            mw.add_include(&method_class_info.class_path());
            let mut code = String::new();
            let argc = if method_name.contains("_init_") {
                args.len() + 1
            } else {
                args.len()
            };
            //println!("argc:{argc}");
            let mut args: Vec<IString> = Vec::with_capacity(argc);
            for _ in 0..argc {
                args.push(mw.vstack_pop().unwrap().1);
            }
            args.reverse();
            let mut args = args.iter();
            let objref = args.next().unwrap();
            if let Some(dep) = ret.dependency() {
                mw.add_include(&dep);
            }
            if *ret == crate::VariableType::Void {
                code.push_str(&format!(
                    "{method_class_name}::{method_name}({objref}",
                    method_class_name = method_class_info.cpp_class()
                ));
            } else {
                let im_name = mw.get_intermidiate();
                code.push_str(&format!(
                    "{ret} {im_name} = {method_class_name}::{method_name}({objref}",
                    ret = ret.c_type(),
                    method_class_name = method_class_info.cpp_class()
                ));
                mw.vstack_push(&im_name, ret.clone());
            }
            for arg in args {
                code.push(',');
                code.push_str(arg);
            }
            code.push_str(");");
            code
        }
        FatOp::InvokeStatic(method_class_info, method_name, args, ret) => {
            mw.add_include(&method_class_info.class_path());
            let mut code = String::new();
            let argc = args.len();
            let mut args: Vec<IString> = Vec::with_capacity(argc);
            for _ in 0..argc {
                args.push(mw.vstack_pop().unwrap().1);
            }
            args.reverse();
            let mut args = args.iter();
            if let Some(dep) = ret.dependency() {
                mw.add_include(&dep);
            }
            if *ret == crate::VariableType::Void {
                code.push_str(&format!(
                    "{method_class_name}::{method_name}(",
                    method_class_name = method_class_info.cpp_class()
                ));
            } else {
                let im_name = mw.get_intermidiate();
                code.push_str(&format!(
                    "{ret} {im_name} = {method_class_name}::{method_name}(",
                    ret = ret.c_type(),
                    method_class_name = method_class_info.cpp_class()
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
        FatOp::Throw => {
            let (exception_type, exception) = mw.vstack_pop().unwrap();
            format!("throw {exception};")
        }
        FatOp::MonitorEnter => {
            let (object_type, object) = mw.vstack_pop().unwrap();
            format!("{object}.monitor_enter();")
        }
        FatOp::MonitorExit => {
            let (object_type, object) = mw.vstack_pop().unwrap();
            format!("{object}.monitor_exit();")
        }
        FatOp::InvokeDynamic => {
            panic!("Invoke Dynamic requires runtime codegen, which is not supported!");
        }
        FatOp::Dup2X2 | &FatOp::MultiANewArray(_, _) | &FatOp::LookupSwitch { .. } => todo!(),
        //_ => todo!("Unsuported op:\"{op:?}\""),
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
    if !method.is_virtual()
        && method.name() == "main__arr_java_cs_lang_cs_String__V"
        && *method.ret_val() == VariableType::Void
        && method.args()
            == &[VariableType::ArrayRef(Box::new(VariableType::ObjectRef(
                ClassInfo::from_java_path("java/lang/String"),
            )))]
    {
        write!(out,"int main(int argc, char** argv){{\n\t//Skip fist exec path\n\targc -= 1;argv += 1;\n\tManagedPointer<RuntimeArray<ManagedPointer<java::lang::String>>> args = managed_from_raw(new RuntimeArray<ManagedPointer<java::lang::String>>(argc));\n\tfor(int arg = 0; arg < argc; arg++){{\n\t\targs->Set(arg,java::lang::String::from_cstring(argv[arg]));\n\t}}\n\t{class_name}::{method_name}(args);\n\treturn 0;\n}}\n",
        class_name = method.class_name(),
        method_name = method.name())?;
    }
    writer.set_sig(&fn_sig);
    writer.add_include(method.class_name());
    if method.is_virtual() {
        writer.push_locals(
            "loc0a",
            &format!(
                "\tManagedPointer<{class}> l0a = managed_from_this({class});\n",
                class = method.class_name()
            ),
        );
    }
    for bb in bb_tree.iter() {
        bb.write(&mut writer);
    }
    out.write_all(writer.final_code().as_bytes())?;
    Ok(())
}

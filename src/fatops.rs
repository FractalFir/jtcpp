use super::{field_descriptor_to_ftype, method_desc_to_args, VariableType};
use crate::importer::{opcodes::OpCode, ImportedJavaClass};
use crate::mangle_method_name;
use crate::IString;
use smallvec::*;
fn fieldref_to_info(index: u16, class: &ImportedJavaClass) -> (VariableType, IString, IString) {
    let (field_class, nametype) = class.lookup_filed_ref(index).unwrap();
    let field_class = class.lookup_class(field_class).unwrap();
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let ftype = field_descriptor_to_ftype(descriptor, class);
    let name = class.lookup_utf8(name).unwrap();
    (ftype, field_class.into(), name.into())
}
fn nametype_to_sig(nametype:u16, class: &ImportedJavaClass)->(IString, Vec<VariableType>, VariableType){
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let name = class.lookup_utf8(name).unwrap();
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    let mangled = mangle_method_name(name, descriptor);
    //let method_id = self.code_container.lookup_or_insert_method(&mangled);
    let (args, ret) = method_desc_to_args(descriptor);
    (mangled, args, ret)
}
fn methodref_to_name_and_sig(
    index: u16,
    class: &ImportedJavaClass,
) -> (IString, IString, Vec<VariableType>, VariableType) {
    let (method_class, nametype) = class.lookup_method_ref(index).unwrap();
    let method_class = class.lookup_class(method_class).unwrap();
    let (mangled, args, ret) = nametype_to_sig(nametype,class);
    //let method_id = self.code_container.lookup_or_insert_method(&mangled);
    (method_class.into(), mangled, args, ret)
}
#[derive(Debug, Clone, PartialEq)]
pub struct ClassInfo {
    cpp_class: IString,
}
impl ClassInfo {
    pub fn from_raw(cpp_class:&str)->Self{
        Self{cpp_class:cpp_class.into()}
    }
    pub fn from_java_path(java_path: &str) -> Self {
        let cpp_class = crate::class::java_class_to_cpp_class(java_path);
        //assert!(!cpp_class.contains('_'),"cpp_class: {java_path} {cpp_class}");
        Self { cpp_class }
    }
    pub fn unknown() -> Self {
        Self {
            cpp_class: "UNKNOWN".into(),
        }
    }
    pub fn is_unknown(&self) -> bool {
        &*self.cpp_class == "UNKNOWN"
    }
    pub fn cpp_class(&self) -> &str {
        &self.cpp_class
    }
    pub fn class_path(&self) -> IString {
        crate::class::cpp_class_to_path(&self.cpp_class)
    }
}
#[derive(Debug, Clone)]
pub(crate) enum DynamicMethodHandle{
    GetField(u16),
    GetStatic(u16),
    PutField(u16),
    PutStatic(u16),
    InvokeVirtual(u16),
    NewInvokeSpecial(u16),
    InvokeStatic(ClassInfo, IString, Box<[VariableType]>, VariableType),
    InvokeSpecial(u16),
    InvokeInterface(u16),
}
impl DynamicMethodHandle{
    fn from_kind(kind:u8,index:u16,jc:&ImportedJavaClass)->Self{
        match kind{
            0=>panic!("Invalid reference kind (0) of a method handle!"),
            1=>Self::GetField(index),
            2=>Self::GetStatic(index),
            3=>Self::PutField(index),
            4=>Self::PutStatic(index),
            5=>Self::InvokeVirtual(index),
            6=>{
                let (class, name, args, ret) = methodref_to_name_and_sig(index, jc);
                let class_info = ClassInfo::from_java_path(&class);
                Self::InvokeStatic(class_info,name,args.into(),ret)
            }
            7=>Self::InvokeSpecial(index),
            8=>Self::NewInvokeSpecial(index),
            9=>Self::InvokeInterface(index),
            9.. =>panic!("Invalid reference kind ({kind}) of a method handle!"),
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) enum FatOp {
    AConstNull,
    IConst(i32),
    BConst(i8),
    SConst(i16),
    LConst(i64),
    StringConst(IString),
    ClassConst(ClassInfo),
    FConst(f32),
    DConst(f64),
    ALoad(u8),
    DLoad(u8),
    FLoad(u8),
    ILoad(u8),
    LLoad(u8),
    AStore(u8),
    DStore(u8),
    FStore(u8),
    IStore(u8),
    LStore(u8),
    DAdd,
    FAdd,
    IAdd,
    LAdd,
    DMul,
    FMul,
    IMul,
    LMul,
    ISub,
    DSub,
    FSub,
    LSub,
    DRem,
    FRem,
    IRem,
    LRem,
    IShr,
    IShl,
    LShr,
    LShl,
    DDiv,
    FDiv,
    IDiv,
    LDiv,
    IAnd,
    LAnd,
    IOr,
    LOr,
    IXOr,
    LXOr,
    INeg,
    LNeg,
    DNeg,
    FNeg,
    LUShr,
    IUShr,
    InvokeSpecial(ClassInfo, IString, Box<[VariableType]>, VariableType),
    InvokeStatic(ClassInfo, IString, Box<[VariableType]>, VariableType),
    InvokeInterface(ClassInfo, IString, Box<[VariableType]>, VariableType), //Unfinshed
    InvokeDynamic(DynamicMethodHandle, IString, Box<[VariableType]>, VariableType), //Temporarly ignored(Hard to parse)
    InvokeVirtual(ClassInfo, IString, Box<[VariableType]>, VariableType),
    ZGetStatic(ClassInfo, IString),
    BGetStatic(ClassInfo, IString),
    SGetStatic(ClassInfo, IString),
    IGetStatic(ClassInfo, IString),
    LGetStatic(ClassInfo, IString),
    FGetStatic(ClassInfo, IString),
    DGetStatic(ClassInfo, IString),
    AGetStatic {
        class_info: ClassInfo,
        static_name: IString,
        type_info: ClassInfo,
    },
    AAGetStatic {
        class_info: ClassInfo,
        static_name: IString,
        atype: VariableType,
    },
    CGetStatic(ClassInfo, IString),
    ZPutStatic(ClassInfo, IString),
    BPutStatic(ClassInfo, IString),
    SPutStatic(ClassInfo, IString),
    IPutStatic(ClassInfo, IString),
    LPutStatic(ClassInfo, IString),
    FPutStatic(ClassInfo, IString),
    DPutStatic(ClassInfo, IString),
    APutStatic {
        class_info: ClassInfo,
        field_name: IString,
        type_info: ClassInfo,
    },
    AAPutStatic {
        class_info: ClassInfo,
        field_name: IString,
        atype: VariableType,
    },
    CPutStatic(ClassInfo, IString),
    ZGetField(ClassInfo, IString),
    BGetField(ClassInfo, IString),
    SGetField(ClassInfo, IString),
    IGetField(ClassInfo, IString),
    LGetField(ClassInfo, IString),
    FGetField(ClassInfo, IString),
    DGetField(ClassInfo, IString),
    AGetField {
        class_info: ClassInfo,
        field_name: IString,
        type_info: ClassInfo,
    },
    CGetField(ClassInfo, IString),
    AAGetField {
        class_info: ClassInfo,
        field_name: IString,
        atype: VariableType,
    },
    ZPutField(ClassInfo, IString),
    BPutField(ClassInfo, IString),
    SPutField(ClassInfo, IString),
    IPutField(ClassInfo, IString),
    LPutField(ClassInfo, IString),
    FPutField(ClassInfo, IString),
    DPutField(ClassInfo, IString),
    APutField {
        class_info: ClassInfo,
        field_name: IString,
        type_info: ClassInfo,
    },
    AAPutField {
        class_info: ClassInfo,
        field_name: IString,
        atype: VariableType,
    },
    CPutField(ClassInfo, IString),
    Dup,
    Dup2,
    DupX1,
    Dup2X1,
    DupX2,
    Dup2X2,
    Swap,
    Pop,
    Pop2,
    Return,
    AReturn,
    FReturn,
    IReturn,
    DReturn,
    LReturn,
    F2D,
    D2F,
    I2L,
    L2I,
    F2L,
    L2F,
    I2F,
    F2I,
    I2C,
    I2S,
    I2B,
    I2D,
    D2I,
    D2L,
    L2D,
    New(ClassInfo),
    ANewArray(ClassInfo),
    MultiANewArray(ClassInfo, u8),
    BNewArray,
    CNewArray,
    DNewArray,
    FNewArray,
    INewArray,
    LNewArray,
    SNewArray,
    ZNewArray,
    CheckedCast(ClassInfo),
    InstanceOf(ClassInfo),
    AAStore,
    BAStore,
    CAStore,
    DAStore,
    FAStore,
    IAStore,
    LAStore,
    SAStore,
    AALoad,
    BALoad,
    CALoad,
    DALoad,
    FALoad,
    IALoad,
    LALoad,
    SALoad,
    LCmp,
    FCmpL,
    FCmpG,
    DCmpL,
    DCmpG,
    ArrayLength,
    IfACmpEq(usize),
    IfICmpGreater(usize),
    IfIGreterEqual(usize),
    IfGreterEqualZero(usize),
    IfGreterZero(usize),
    IfLessZero(usize),
    IfLessEqualZero(usize),
    IfNull(usize),
    IfNotNull(usize),
    IfZero(usize),
    IfICmpNe(usize),
    IfICmpEq(usize),
    IfACmpNe(usize),
    IfICmpLessEqual(usize),
    IfICmpLess(usize),
    GoTo(usize),
    IInc(u8, i8),
    Throw,
    MonitorEnter,
    MonitorExit,
    LookupSwitch {
        default_op: usize,
        pairs: Box<[(i32, usize)]>,
    },
    //Emmited after split into basic blocks, for handling exceptions
    //BeginTry,
    //EndTry,
    //Catch,
}
impl FatOp {
    pub fn jump_target(&self) -> Option<SmallVec<[usize; 4]>> {
        match self {
            Self::IfACmpEq(target) => Some(smallvec![*target]),
            Self::IfICmpGreater(target) => Some(smallvec![*target]),
            Self::IfIGreterEqual(target) => Some(smallvec![*target]),
            Self::IfGreterEqualZero(target) => Some(smallvec![*target]),
            Self::IfGreterZero(target) => Some(smallvec![*target]),
            Self::IfLessZero(target) => Some(smallvec![*target]),
            Self::IfLessEqualZero(target) => Some(smallvec![*target]),
            Self::IfNull(target) => Some(smallvec![*target]),
            Self::IfNotNull(target) => Some(smallvec![*target]),
            Self::IfZero(target) => Some(smallvec![*target]),
            Self::IfICmpNe(target) => Some(smallvec![*target]),
            Self::IfICmpEq(target) => Some(smallvec![*target]),
            Self::IfACmpNe(target) => Some(smallvec![*target]),
            Self::IfICmpLessEqual(target) => Some(smallvec![*target]),
            Self::IfICmpLess(target) => Some(smallvec![*target]),
            Self::GoTo(target) => Some(smallvec![*target]),
            Self::LookupSwitch { default_op, pairs } => {
                let mut sv = smallvec![*default_op];
                for (_key, target) in pairs.iter() {
                    sv.push(*target);
                }
                Some(sv)
            }
            _ => None,
        }
    }
}
pub(crate) fn find_op_with_offset(ops: &[(OpCode, u16)], idx: u16) -> Option<usize> {
    for (current, op) in ops.iter().enumerate() {
        if op.1 == idx {
            return Some(current);
        }
    }
    None
}
pub(crate) fn expand_ops(ops: &[(OpCode, u16)], class: &ImportedJavaClass) -> Box<[FatOp]> {
    let mut fatops = Vec::with_capacity(ops.len());
    for op in ops {
        let cop = match &op.0 {
            OpCode::LoadConst(index) => {
                let const_item = class.lookup_item(*index).unwrap();
                match const_item {
                    crate::importer::ConstantItem::ConstString { string_index } => {
                        let string = class
                            .lookup_utf8(*string_index)
                            .unwrap()
                            .replace("\n", "\\n")
                            .replace("\r", "\\r");
                        FatOp::StringConst(string.into())
                    }
                    crate::importer::ConstantItem::Class { name_index } => {
                        let class_name = class.lookup_utf8(*name_index).unwrap();
                        FatOp::ClassConst(ClassInfo::from_java_path(class_name))
                    }
                    crate::importer::ConstantItem::Float(float) => FatOp::FConst(*float),
                    crate::importer::ConstantItem::Double(double) => FatOp::DConst(*double),
                    crate::importer::ConstantItem::Intiger(int) => FatOp::IConst(*int),
                    crate::importer::ConstantItem::Long(long) => FatOp::LConst(*long),
                    _ => todo!("can't handle const!{const_item:?}"),
                }
            }
            OpCode::AConstNull => FatOp::AConstNull,
            OpCode::BIPush(value) => FatOp::BConst(*value),
            OpCode::SIPush(value) => FatOp::SConst(*value),
            OpCode::IConst(int) => FatOp::IConst(*int),
            OpCode::FConst(float) => FatOp::FConst(*float),
            OpCode::DConst(double) => FatOp::DConst(*double),
            OpCode::LConst(long) => FatOp::LConst(*long),
            OpCode::LCmp => FatOp::LCmp,
            OpCode::FCmpG => FatOp::FCmpG,
            OpCode::FCmpL => FatOp::FCmpL,
            OpCode::DCmpL => FatOp::DCmpL,
            OpCode::DCmpG => FatOp::DCmpG,
            OpCode::F2D => FatOp::F2D,
            OpCode::D2F => FatOp::D2F,
            OpCode::ISub => FatOp::ISub,
            OpCode::DSub => FatOp::DSub,
            OpCode::FSub => FatOp::FSub,
            OpCode::LSub => FatOp::LSub,
            OpCode::DAdd => FatOp::DAdd,
            OpCode::FAdd => FatOp::FAdd,
            OpCode::IAdd => FatOp::IAdd,
            OpCode::LAdd => FatOp::LAdd,
            OpCode::DMul => FatOp::DMul,
            OpCode::FMul => FatOp::FMul,
            OpCode::IMul => FatOp::IMul,
            OpCode::LMul => FatOp::LMul,
            OpCode::DDiv => FatOp::DDiv,
            OpCode::FDiv => FatOp::FDiv,
            OpCode::IDiv => FatOp::IDiv,
            OpCode::LDiv => FatOp::LDiv,
            OpCode::DRem => FatOp::DRem,
            OpCode::FRem => FatOp::FRem,
            OpCode::IRem => FatOp::IRem,
            OpCode::LRem => FatOp::LRem,
            OpCode::IShr => FatOp::IShr,
            OpCode::IShl => FatOp::IShl,
            OpCode::LShl => FatOp::LShl,
            OpCode::LUShr => FatOp::LUShr,
            OpCode::LShr => FatOp::LShr,
            OpCode::IAnd => FatOp::IAnd,
            OpCode::LAnd => FatOp::LAnd,
            OpCode::IOr => FatOp::IOr,
            OpCode::LOr => FatOp::LOr,
            OpCode::IXOr => FatOp::IXOr,
            OpCode::LXOr => FatOp::LXOr,
            OpCode::INeg => FatOp::INeg,
            OpCode::LNeg => FatOp::LNeg,
            OpCode::DNeg => FatOp::DNeg,
            OpCode::FNeg => FatOp::FNeg,
            OpCode::L2I => FatOp::L2I,
            OpCode::L2F => FatOp::L2F,
            OpCode::F2I => FatOp::F2I,
            OpCode::F2L => FatOp::F2L,
            OpCode::I2B => FatOp::I2B,
            OpCode::I2C => FatOp::I2C,
            OpCode::I2F => FatOp::I2F,
            OpCode::I2D => FatOp::I2D,
            OpCode::I2S => FatOp::I2S,
            OpCode::I2L => FatOp::I2L,
            OpCode::D2I => FatOp::D2I,
            OpCode::D2L => FatOp::D2L,
            OpCode::L2D => FatOp::L2D,
            OpCode::ALoad(index) => FatOp::ALoad(*index),
            OpCode::ILoad(index) => FatOp::ILoad(*index),
            OpCode::LLoad(index) => FatOp::LLoad(*index),
            OpCode::AStore(index) => FatOp::AStore(*index),
            OpCode::DStore(index) => FatOp::DStore(*index),
            OpCode::FStore(index) => FatOp::FStore(*index),
            OpCode::IStore(index) => FatOp::IStore(*index),
            OpCode::LStore(index) => FatOp::LStore(*index),
            OpCode::FLoad(index) => FatOp::FLoad(*index),
            OpCode::DLoad(index) => FatOp::DLoad(*index),
            OpCode::GetStatic(index) => {
                let (ftype, class_name, static_name) = fieldref_to_info(*index, class);
                let class_info = ClassInfo::from_java_path(&class_name);
                match ftype {
                    VariableType::Bool => FatOp::ZGetStatic(class_info, static_name),
                    VariableType::Byte => FatOp::BGetStatic(class_info, static_name),
                    VariableType::Short => FatOp::SGetStatic(class_info, static_name),
                    VariableType::Char => FatOp::CGetStatic(class_info, static_name),
                    VariableType::Int => FatOp::IGetStatic(class_info, static_name),
                    VariableType::Long => FatOp::LGetStatic(class_info, static_name),
                    VariableType::Float => FatOp::FGetStatic(class_info, static_name),
                    VariableType::Double => FatOp::DGetStatic(class_info, static_name),
                    VariableType::ObjectRef(type_class_info) => FatOp::AGetStatic {
                        class_info,
                        static_name,
                        type_info: type_class_info,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAGetStatic {
                        class_info,
                        static_name,
                        atype: *atype,
                    },
                    VariableType::Void => panic!("ERR: GetStatic op with invalid field type Void!"),
                }
            }
            OpCode::PutStatic(index) => {
                let (ftype, class_name, field_name) = fieldref_to_info(*index, class);
                let class_info = ClassInfo::from_java_path(&class_name);
                match ftype {
                    VariableType::Bool => FatOp::ZPutStatic(class_info, field_name),
                    VariableType::Byte => FatOp::BPutStatic(class_info, field_name),
                    VariableType::Short => FatOp::SPutStatic(class_info, field_name),
                    VariableType::Char => FatOp::CPutStatic(class_info, field_name),
                    VariableType::Int => FatOp::IPutStatic(class_info, field_name),
                    VariableType::Long => FatOp::LPutStatic(class_info, field_name),
                    VariableType::Float => FatOp::FPutStatic(class_info, field_name),
                    VariableType::Double => FatOp::DPutStatic(class_info, field_name),
                    VariableType::ObjectRef(type_class_info) => FatOp::APutStatic {
                        class_info,
                        field_name,
                        type_info: type_class_info,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAPutStatic {
                        class_info,
                        field_name,
                        atype: *atype,
                    },
                    VariableType::Void => panic!("ERR: PutStatic op with invalid field type Void!"),
                }
            }
            OpCode::GetField(index) => {
                let (ftype, class_name, field_name) = fieldref_to_info(*index, class);
                let class_info = ClassInfo::from_java_path(&class_name);
                match ftype {
                    VariableType::Bool => FatOp::ZGetField(class_info, field_name),
                    VariableType::Byte => FatOp::BGetField(class_info, field_name),
                    VariableType::Short => FatOp::SGetField(class_info, field_name),
                    VariableType::Char => FatOp::CGetField(class_info, field_name),
                    VariableType::Int => FatOp::IGetField(class_info, field_name),
                    VariableType::Long => FatOp::LGetField(class_info, field_name),
                    VariableType::Float => FatOp::FGetField(class_info, field_name),
                    VariableType::Double => FatOp::DGetField(class_info, field_name),
                    VariableType::ObjectRef(type_class_info) => FatOp::AGetField {
                        class_info,
                        field_name,
                        type_info: type_class_info,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAGetField {
                        class_info,
                        field_name,
                        atype: *atype,
                    },
                    VariableType::Void => panic!("ERR: GetField op with invalid field type Void!"),
                }
            }
            OpCode::IfICmpEq(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfICmpEq(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfNull(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfNull(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfNotNull(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfNotNull(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfNotZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfICmpNe(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfICmpNe(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfACmpEq(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfACmpEq(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfIGreterEqual(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfIGreterEqual(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfGreterEqualZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfGreterEqualZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfGreterZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfGreterZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfICmpGreater(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfICmpGreater(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfLessZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfLessZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfICmpLessEqual(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfICmpLessEqual(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfICmpLessThan(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfICmpLess(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfLessEqualZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfLessEqualZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfACmpNe(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::IfACmpNe(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::GoTo(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + *op_offset as i32) as u16;
                FatOp::GoTo(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::PutField(index) => {
                let (ftype, class_name, field_name) = fieldref_to_info(*index, class);
                let class_info = ClassInfo::from_java_path(&class_name);
                match ftype {
                    VariableType::Bool => FatOp::ZPutField(class_info, field_name),
                    VariableType::Byte => FatOp::BPutField(class_info, field_name),
                    VariableType::Short => FatOp::SPutField(class_info, field_name),
                    VariableType::Char => FatOp::CPutField(class_info, field_name),
                    VariableType::Int => FatOp::IPutField(class_info, field_name),
                    VariableType::Long => FatOp::LPutField(class_info, field_name),
                    VariableType::Float => FatOp::FPutField(class_info, field_name),
                    VariableType::Double => FatOp::DPutField(class_info, field_name),
                    VariableType::ObjectRef(type_class_info) => FatOp::APutField {
                        class_info,
                        field_name,
                        type_info: type_class_info,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAPutField {
                        class_info,
                        field_name,
                        atype: *atype,
                    },
                    VariableType::Void => panic!("ERR: PutField op with invalid field type Void!"),
                }
            }
            OpCode::New(index) => {
                let class_name = class.lookup_class(*index).unwrap();
                FatOp::New(ClassInfo::from_java_path(class_name))
            }
            OpCode::ANewArray(index) => {
                let class_name = class.lookup_class(*index).unwrap();
                FatOp::ANewArray(ClassInfo::from_java_path(&class_name))
            }
            OpCode::MultiANewArray(index, dimensions) => {
                let class_name = class.lookup_class(*index).unwrap();
                let class_info = ClassInfo::from_java_path(&class_name);
                FatOp::MultiANewArray(class_info, *dimensions)
            }
            OpCode::NewArray(typeid) => match *typeid {
                4 => FatOp::ZNewArray,
                5 => FatOp::CNewArray,
                6 => FatOp::FNewArray,
                7 => FatOp::DNewArray,
                8 => FatOp::BNewArray,
                9 => FatOp::SNewArray,
                10 => FatOp::INewArray,
                11 => FatOp::LNewArray,
                0..=3 | 11.. => panic!("Invalid type ID in NewArray Op!"),
            },
            OpCode::CheckCast(index) => {
                let class_name = class.lookup_class(*index).unwrap();
                let class_info = ClassInfo::from_java_path(&class_name);
                FatOp::CheckedCast(class_info)
            }
            OpCode::InstanceOf(index) => {
                let class_name = class.lookup_class(*index).unwrap();
                let class_info = ClassInfo::from_java_path(&class_name);
                FatOp::InstanceOf(class_info)
            }
            OpCode::Swap => FatOp::Swap,
            OpCode::Dup => FatOp::Dup,
            OpCode::Dup2 => FatOp::Dup2,
            OpCode::DupX1 => FatOp::DupX1,
            OpCode::Dup2X1 => FatOp::Dup2X1,
            OpCode::DupX2 => FatOp::DupX2,
            OpCode::Dup2X2 => FatOp::Dup2X2,
            OpCode::Pop => FatOp::Pop,
            OpCode::Pop2 => FatOp::Pop2,
            // TODO: handle non-static methods(change argc by 1)
            OpCode::InvokeSpecial(index) => {
                let (method_class, method_name, args, ret) =
                    methodref_to_name_and_sig(*index, class);
                let class_info = ClassInfo::from_java_path(&method_class);
                let method_name: IString = method_name
                    .replace("<init>", "_init_")
                    .replace("<cinit>", "_cinit_")
                    .into();
                if method_name.contains("_init_") || method_name.contains("_cinit_") {
                    FatOp::InvokeSpecial(class_info, method_name, args.into(), ret)
                } else {
                    FatOp::InvokeVirtual(class_info, method_name, args.into(), ret)
                }
            }
            OpCode::InvokeStatic(index) => {
                let (method_class_name, name, args, ret) = methodref_to_name_and_sig(*index, class);
                let class_info = ClassInfo::from_java_path(&method_class_name);
                FatOp::InvokeStatic(class_info, name, args.into(), ret)
            }
            OpCode::InvokeVirtual(index) => {
                let (class, name, args, ret) = methodref_to_name_and_sig(*index, class);
                let class_info = ClassInfo::from_java_path(&class);
                FatOp::InvokeVirtual(class_info, name, args.into(), ret)
            }
            OpCode::InvokeInterface(index) => {
                let (class, name, args, ret) = methodref_to_name_and_sig(*index, class);
                let class_info = ClassInfo::from_java_path(&class);
                FatOp::InvokeInterface(class_info, name, args.into(), ret)
            }
            OpCode::InvokeDynamic(index) => {
                let (bootstrap_method_attr_index, name_and_type_index) =
                    class.lookup_invoke_dynamic(*index).unwrap();
                
                let bootstrap_method = class
                    .lookup_bootstrap_method(bootstrap_method_attr_index)
                    .unwrap();
                let (reference_kind, reference_index) = class
                    .lookup_method_handle(bootstrap_method.bootstrap_method_ref)
                    .unwrap();
                let handle = DynamicMethodHandle::from_kind(reference_kind, reference_index, class);
                println!("\nhandle:{handle:?},reference_index:{reference_index}\n");
                //let (name, argc) = methodref_to_mangled_and_argc(bootstrap_method.bootstrap_method_ref, class);
                let (name,args,ret) = nametype_to_sig(name_and_type_index, class);
                FatOp::InvokeDynamic(handle,name,args.into(),ret)
                //FatOp::InvokeDynamic(name, argc)
            }
            OpCode::Return => FatOp::Return,
            OpCode::AReturn => FatOp::AReturn,
            OpCode::FReturn => FatOp::FReturn,
            OpCode::IReturn => FatOp::IReturn,
            OpCode::LReturn => FatOp::LReturn,
            OpCode::DReturn => FatOp::DReturn,
            OpCode::AAStore => FatOp::AAStore,
            OpCode::BAStore => FatOp::BAStore,
            OpCode::CAStore => FatOp::CAStore,
            OpCode::DAStore => FatOp::DAStore,
            OpCode::FAStore => FatOp::FAStore,
            OpCode::IAStore => FatOp::IAStore,
            OpCode::LAStore => FatOp::LAStore,
            OpCode::SAStore => FatOp::SAStore,
            //OpCode::ZAStore => FatOp::ZAStore,
            OpCode::AALoad => FatOp::AALoad,
            OpCode::BALoad => FatOp::BALoad,
            OpCode::CALoad => FatOp::CALoad,
            OpCode::DALoad => FatOp::DALoad,
            OpCode::FALoad => FatOp::FALoad,
            OpCode::IALoad => FatOp::IALoad,
            OpCode::LALoad => FatOp::LALoad,
            OpCode::SALoad => FatOp::SALoad,
            OpCode::ArrayLength => FatOp::ArrayLength,
            OpCode::IInc(local, offset) => FatOp::IInc(*local, *offset),
            OpCode::Throw => FatOp::Throw,
            OpCode::MonitorEnter => FatOp::MonitorEnter,
            OpCode::MonitorExit => FatOp::MonitorExit,
            OpCode::LookupSwitch(switch) => {
                let default_offset: u16 = (op.1 as i32 + switch.default_offset) as u16;
                let default_op = find_op_with_offset(ops, default_offset).unwrap();
                let mut pairs = Vec::with_capacity(switch.pairs.len());
                for (key, offset) in switch.pairs.iter() {
                    let offset: u16 = (op.1 as i32 + *offset) as u16;
                    let op = find_op_with_offset(ops, offset).unwrap();
                    pairs.push((*key, op));
                }
                FatOp::LookupSwitch {
                    default_op,
                    pairs: pairs.into(),
                }
            }
            _ => todo!("can't expand op {op:?}"),
        };
        fatops.push(cop);
    }
    fatops.into()
}

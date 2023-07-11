use super::{
    class_path_to_class_mangled, field_descriptor_to_ftype, method_desc_to_args, VariableType,
};
use crate::importer::{opcodes::OpCode, ImportedJavaClass};
use crate::IString;
use crate::{mangle_method_name, mangle_method_name_partial, method_desc_to_argc};
use smallvec::*;
fn fieldref_to_info(index: u16, class: &ImportedJavaClass) -> (VariableType, IString, IString) {
    let (field_class, nametype) = class.lookup_filed_ref(index).unwrap();
    let field_class = class.lookup_class(field_class).unwrap();
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let ftype = field_descriptor_to_ftype(descriptor, class);
    let name = class.lookup_utf8(name).unwrap();
    (ftype, field_class.into(), name.into())
}
fn methodref_to_mangled_and_argc(index: u16, class: &ImportedJavaClass) -> (IString, u8) {
    let (method_class, nametype) = class.lookup_method_ref(index).unwrap();
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let _method_class = class.lookup_class(method_class).unwrap();
    let name = class.lookup_utf8(name).unwrap();
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    let mangled = mangle_method_name(name, descriptor);
    //let method_id = self.code_container.lookup_or_insert_method(&mangled);
    let argc = method_desc_to_argc(&descriptor);
    (mangled, argc)
}
fn methodref_to_mangled_and_sig(
    index: u16,
    class: &ImportedJavaClass,
) -> (IString, IString, Vec<VariableType>, VariableType) {
    let (method_class, nametype) = class.lookup_method_ref(index).unwrap();
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let method_class = class.lookup_class(method_class).unwrap();
    let name = class.lookup_utf8(name).unwrap();
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    let mangled = mangle_method_name(name, descriptor);
    //let method_id = self.code_container.lookup_or_insert_method(&mangled);
    let (args, ret) = method_desc_to_args(descriptor);
    (method_class.into(), mangled, args, ret)
}
fn methodref_to_partial_mangled_and_argc(
    index: u16,
    class: &ImportedJavaClass,
) -> (IString, IString, u8) {
    let (method_class, nametype) = class.lookup_method_ref(index).unwrap();
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let method_class = class.lookup_class(method_class).unwrap();
    let name = class.lookup_utf8(name).unwrap();
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    let mangled = mangle_method_name_partial(name, descriptor);
    //let method_id = self.code_container.lookup_or_insert_method(&mangled);
    let argc = method_desc_to_argc(&descriptor);
    (method_class.into(), mangled, argc)
}
#[derive(Debug, Clone)]
pub(crate) enum FatOp {
    AConstNull,
    IConst(i32),
    BConst(i8),
    SConst(i16),
    LConst(i64),
    StringConst(IString),
    ClassConst(IString),
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
    LShr,
    IShl,
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
    InvokeSpecial(IString, IString, u8),
    InvokeStatic(IString, IString, Box<[VariableType]>, VariableType),
    InvokeInterface(IString, u8),
    InvokeDynamic, //Temporarly ignored(Hard to parse)
    InvokeVirtual(IString, IString, Box<[VariableType]>, VariableType),
    ZGetStatic(IString, IString),
    BGetStatic(IString, IString),
    SGetStatic(IString, IString),
    IGetStatic(IString, IString),
    LGetStatic(IString, IString),
    FGetStatic(IString, IString),
    DGetStatic(IString, IString),
    AGetStatic {
        class_name: IString,
        static_name: IString,
        type_name: IString,
    },
    AAGetStatic {
        //class_name: IString,
        //field_name: IString,
        atype: VariableType,
    },
    CGetStatic(IString, IString),
    ZPutStatic(IString, IString),
    BPutStatic(IString, IString),
    SPutStatic(IString, IString),
    IPutStatic(IString, IString),
    LPutStatic(IString, IString),
    FPutStatic(IString, IString),
    DPutStatic(IString, IString),
    APutStatic {
        class_name: IString,
        field_name: IString,
        type_name: IString,
    },
    AAPutStatic {
        atype: VariableType,
    },
    CPutStatic(IString, IString),
    ZGetField(IString, IString),
    BGetField(IString, IString),
    SGetField(IString, IString),
    IGetField(IString, IString),
    LGetField(IString, IString),
    FGetField(IString, IString),
    DGetField(IString, IString),
    AGetField {
        class_name: IString,
        field_name: IString,
        type_name: IString,
    },
    CGetField(IString, IString),
    AAGetField {
        class_name: IString,
        field_name: IString,
        atype: VariableType,
    },
    ZPutField(IString, IString),
    BPutField(IString, IString),
    SPutField(IString, IString),
    IPutField(IString, IString),
    LPutField(IString, IString),
    FPutField(IString, IString),
    DPutField(IString, IString),
    APutField {
        class_name: IString,
        field_name: IString,
        type_name: IString,
    },
    AAPutField {
        class_name: IString,
        field_name: IString,
        atype: VariableType,
    },
    CPutField(IString, IString),
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
    New(IString),
    ANewArray(IString),
    MultiANewArray(IString, u8),
    BNewArray,
    CNewArray,
    DNewArray,
    FNewArray,
    INewArray,
    LNewArray,
    SNewArray,
    ZNewArray,
    CheckedCast(IString),
    InstanceOf(IString),
    AAStore,
    BAStore,
    CAStore,
    DAStore,
    FAStore,
    IAStore,
    LAStore,
    SAStore,
    ZAStore,
    AALoad,
    BALoad,
    CALoad,
    DALoad,
    FALoad,
    IALoad,
    LALoad,
    SALoad,
    ZALoad,
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
    IfNotZero(usize),
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
            Self::IfNotZero(target) => Some(smallvec![*target]),
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
                        let string = class.lookup_utf8(*string_index).unwrap();
                        FatOp::StringConst(string.into())
                    }
                    crate::importer::ConstantItem::Class { name_index } => {
                        let class_name = class.lookup_utf8(*name_index).unwrap();
                        FatOp::ClassConst(class_name.into())
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
            OpCode::LShr => FatOp::LShr,
            OpCode::IShl => FatOp::IShl,
            OpCode::LShl => FatOp::LShl,
            OpCode::LUShr => FatOp::LUShr,
            OpCode::IUShr => FatOp::IUShr,
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
                match ftype {
                    VariableType::Bool => FatOp::ZGetStatic(class_name, static_name),
                    VariableType::Byte => FatOp::BGetStatic(class_name, static_name),
                    VariableType::Short => FatOp::SGetStatic(class_name, static_name),
                    VariableType::Char => FatOp::CGetStatic(class_name, static_name),
                    VariableType::Int => FatOp::IGetStatic(class_name, static_name),
                    VariableType::Long => FatOp::LGetStatic(class_name, static_name),
                    VariableType::Float => FatOp::FGetStatic(class_name, static_name),
                    VariableType::Double => FatOp::DGetStatic(class_name, static_name),
                    VariableType::ObjectRef { name } => FatOp::AGetStatic {
                        class_name,
                        static_name,
                        type_name: name,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAGetStatic { atype: *atype },
                    VariableType::Void => panic!("ERR: GetStatic op with invalid field type Void!"),
                }
            }
            OpCode::PutStatic(index) => {
                let (ftype, class_name, field_name) = fieldref_to_info(*index, class);
                match ftype {
                    VariableType::Bool => FatOp::ZPutStatic(class_name, field_name),
                    VariableType::Byte => FatOp::BPutStatic(class_name, field_name),
                    VariableType::Short => FatOp::SPutStatic(class_name, field_name),
                    VariableType::Char => FatOp::CPutStatic(class_name, field_name),
                    VariableType::Int => FatOp::IPutStatic(class_name, field_name),
                    VariableType::Long => FatOp::LPutStatic(class_name, field_name),
                    VariableType::Float => FatOp::FPutStatic(class_name, field_name),
                    VariableType::Double => FatOp::DPutStatic(class_name, field_name),
                    VariableType::ObjectRef { name } => FatOp::APutStatic {
                        class_name,
                        field_name,
                        type_name: name,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAPutStatic { atype: *atype },
                    VariableType::Void => panic!("ERR: PutStatic op with invalid field type Void!"),
                }
            }
            OpCode::GetField(index) => {
                let (ftype, class_name, field_name) = fieldref_to_info(*index, class);
                match ftype {
                    VariableType::Bool => FatOp::ZGetField(class_name, field_name),
                    VariableType::Byte => FatOp::BGetField(class_name, field_name),
                    VariableType::Short => FatOp::SGetField(class_name, field_name),
                    VariableType::Char => FatOp::CGetField(class_name, field_name),
                    VariableType::Int => FatOp::IGetField(class_name, field_name),
                    VariableType::Long => FatOp::LGetField(class_name, field_name),
                    VariableType::Float => FatOp::FGetField(class_name, field_name),
                    VariableType::Double => FatOp::DGetField(class_name, field_name),
                    VariableType::ObjectRef { name } => FatOp::AGetField {
                        class_name,
                        field_name,
                        type_name: name,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAGetField {
                        class_name,
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
                let class_name = class_path_to_class_mangled(&class_name);
                match ftype {
                    VariableType::Bool => FatOp::ZPutField(class_name, field_name),
                    VariableType::Byte => FatOp::BPutField(class_name, field_name),
                    VariableType::Short => FatOp::SPutField(class_name, field_name),
                    VariableType::Char => FatOp::CPutField(class_name, field_name),
                    VariableType::Int => FatOp::IPutField(class_name, field_name),
                    VariableType::Long => FatOp::LPutField(class_name, field_name),
                    VariableType::Float => FatOp::FPutField(class_name, field_name),
                    VariableType::Double => FatOp::DPutField(class_name, field_name),
                    VariableType::ObjectRef { name } => FatOp::APutField {
                        class_name,
                        field_name,
                        type_name: name,
                    },
                    VariableType::ArrayRef(atype) => FatOp::AAPutField {
                        class_name,
                        field_name,
                        atype: *atype,
                    },
                    VariableType::Void => panic!("ERR: PutField op with invalid field type Void!"),
                }
            }
            OpCode::New(index) => {
                let class_name = class.lookup_class(*index).unwrap();
                FatOp::New(class_path_to_class_mangled(class_name.into()))
            }
            OpCode::ANewArray(index) => {
                let class_name = class.lookup_class(*index).unwrap();
                FatOp::ANewArray(class_name.into())
            }
            OpCode::MultiANewArray(index, dimensions) => {
                let class_name = class.lookup_class(*index).unwrap();
                FatOp::MultiANewArray(class_name.into(), *dimensions)
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
                FatOp::CheckedCast(class_name.into())
            }
            OpCode::InstanceOf(index) => {
                let class_name = class.lookup_class(*index).unwrap();
                FatOp::InstanceOf(class_name.into())
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
            ///TODO: handle non-static methods(change argc by 1)
            OpCode::InvokeSpecial(index) => {
                let (method_class, method_name, args, ret) =
                    methodref_to_mangled_and_sig(*index, class);
                //let (method_class, _) = class.lookup_method_ref(*index).unwrap();
                // let method_class = class.lookup_class(method_class).unwrap();
                // let method_class = class_path_to_class_mangled(method_class);
                // Either <init> or <cinit>
                if method_name.contains("_init_") {
                    //First arg is already provided by virtual
                    //args.remove(0);
                    FatOp::InvokeVirtual(method_class, method_name, args.into(), ret)
                } else {
                    FatOp::InvokeSpecial(method_class, method_name, args.len() as u8)
                }
            }
            OpCode::InvokeStatic(index) => {
                let (method_class_name, name, args, ret) =
                    methodref_to_mangled_and_sig(*index, class);
                FatOp::InvokeStatic(method_class_name, name, args.into(), ret)
            }
            OpCode::InvokeVirtual(index) => {
                let (_, _, args, ret) = methodref_to_mangled_and_sig(*index, class);
                let (class, name, _argc) = methodref_to_partial_mangled_and_argc(*index, class);
                FatOp::InvokeVirtual(class, name, args.into(), ret)
            }
            OpCode::InvokeInterface(index) => {
                let (name, argc) = methodref_to_mangled_and_argc(*index, class);
                //TODO:Potentially handle static interface methods.
                FatOp::InvokeInterface(name, argc + 1)
            }
            OpCode::InvokeDynamic(index) => {
                let (bootstrap_method_attr_index, _name_and_type_index) =
                    class.lookup_invoke_dynamic(*index).unwrap();
                let bootstrap_method = class
                    .lookup_bootstrap_method(bootstrap_method_attr_index)
                    .unwrap();
                let (_reference_kind, _reference_index) = class
                    .lookup_method_handle(bootstrap_method.bootstrap_method_ref)
                    .unwrap();
                //println!("reference_kind:{reference_kind},reference_index:{reference_index}");
                //let (name, argc) = methodref_to_mangled_and_argc(bootstrap_method.bootstrap_method_ref, class);
                FatOp::InvokeDynamic
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
                let default_offset: u16 = (op.1 as i32 + switch.default_offset as i32) as u16;
                let default_op = find_op_with_offset(ops, default_offset).unwrap();
                let mut pairs = Vec::with_capacity(switch.pairs.len());
                for (key, offset) in switch.pairs.iter() {
                    let offset: u16 = (op.1 as i32 + *offset as i32) as u16;
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
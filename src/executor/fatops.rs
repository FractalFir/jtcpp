use super::{field_descriptor_to_ftype, FieldType};
use crate::importer::{opcodes::OpCode, ImportedJavaClass};
use crate::IString;
use crate::{mangle_method_name, mangle_method_name_partial, method_desc_to_argc};
fn fieldref_to_info(index: u16, class: &ImportedJavaClass) -> (FieldType, IString, IString) {
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
    let method_class = class.lookup_class(method_class).unwrap();
    let name = class.lookup_utf8(name).unwrap();
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    let mangled = mangle_method_name(method_class, name, descriptor);
    //let method_id = self.code_container.lookup_or_insert_method(&mangled);
    let argc = method_desc_to_argc(&descriptor);
    (mangled, argc)
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
    IMul,
    FMul,
    ISub,
    DSub,
    FSub,
    LSub,
    IRem,
    IShr,
    LShr,
    IShl,
    LShl,
    IDiv,
    FDiv,
    IAnd,
    LAnd,
    IOr,
    LOr,
    IXOr,
    LXOr,
    INeg,
    LUShr,
    LUShl,
    InvokeSpecial(IString, u8),
    InvokeStatic(IString, u8),
    InvokeInterface(IString, u8),
    InvokeDynamic, //Temporarly ignored(Hard to parse)
    InvokeVirtual(IString, IString, u8),
    ZGetStatic(IString, IString),
    BGetStatic(IString, IString),
    SGetStatic(IString, IString),
    IGetStatic(IString, IString),
    LGetStatic(IString, IString),
    FGetStatic(IString, IString),
    DGetStatic(IString, IString),
    AGetStatic(IString, IString),
    CGetStatic(IString, IString),
    ZPutStatic(IString, IString),
    BPutStatic(IString, IString),
    SPutStatic(IString, IString),
    IPutStatic(IString, IString),
    LPutStatic(IString, IString),
    FPutStatic(IString, IString),
    DPutStatic(IString, IString),
    APutStatic(IString, IString),
    CPutStatic(IString, IString),
    ZGetField(IString, IString),
    BGetField(IString, IString),
    SGetField(IString, IString),
    IGetField(IString, IString),
    LGetField(IString, IString),
    FGetField(IString, IString),
    DGetField(IString, IString),
    AGetField(IString, IString),
    CGetField(IString, IString),
    ZPutField(IString, IString),
    BPutField(IString, IString),
    SPutField(IString, IString),
    IPutField(IString, IString),
    LPutField(IString, IString),
    FPutField(IString, IString),
    DPutField(IString, IString),
    APutField(IString, IString),
    CPutField(IString, IString),
    Dup,
    Dup2,
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
    New(IString),
    ANewArray(IString),
    CheckedCast(IString),
    InstanceOf(IString),
    AAStore,
    AALoad,
    BALoad,
    CALoad,
    DALoad,
    FALoad,
    IALoad,
    LALoad,
    LCmp,
    ArrayLength,
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
        let cop = match op.0 {
            OpCode::LoadConst(index) => {
                let const_item = class.lookup_item(index).unwrap();
                match const_item {
                    crate::importer::ConstantItem::ConstString { string_index } => {
                        let string = class.lookup_utf8(*string_index).unwrap();
                        FatOp::StringConst(string.into())
                    }
                    crate::importer::ConstantItem::Class { name_index } => {
                        let class_name = class.lookup_utf8(*name_index).unwrap();
                        FatOp::ClassConst(class_name.into())
                    }
                    _ => todo!("can't handle const!{const_item:?}"),
                }
            }
            OpCode::AConstNull => FatOp::AConstNull,
            OpCode::BIPush(value) => FatOp::BConst(value),
            OpCode::SIPush(value) => FatOp::SConst(value),
            OpCode::IConst(int) => FatOp::IConst(int),
            OpCode::FConst(float) => FatOp::FConst(float),
            OpCode::LConst(long) => FatOp::LConst(long),
            OpCode::LCmp => FatOp::LCmp,
            OpCode::F2D => FatOp::F2D,
            OpCode::D2F => FatOp::D2F,
            OpCode::ISub => FatOp::ISub,
            OpCode::DSub => FatOp::DSub,
            OpCode::FSub => FatOp::FSub,
            OpCode::LSub => FatOp::LSub,
            OpCode::IAdd => FatOp::IAdd,
            OpCode::FAdd => FatOp::FAdd,
            OpCode::LAdd => FatOp::LAdd,
            OpCode::IMul => FatOp::IMul,
            OpCode::FMul => FatOp::FMul,
            OpCode::IDiv => FatOp::IDiv,
            OpCode::FDiv => FatOp::FDiv,
            OpCode::IRem => FatOp::IRem,
            OpCode::IShr => FatOp::IShr,
            OpCode::LShr => FatOp::LShr,
            OpCode::IShl => FatOp::IShl,
            OpCode::LShl => FatOp::LShl,
            OpCode::LUShr => FatOp::LUShr,
            OpCode::LUShl => FatOp::LUShl,
            OpCode::IAnd => FatOp::IAnd,
            OpCode::LAnd => FatOp::LAnd,
            OpCode::IOr => FatOp::IOr,
            OpCode::LOr => FatOp::LOr,
            OpCode::IXOr => FatOp::IXOr,
            OpCode::LXOr => FatOp::LXOr,
            OpCode::INeg => FatOp::INeg,
            OpCode::I2L => FatOp::I2L,
            OpCode::L2I => FatOp::L2I,
            OpCode::ALoad(index) => FatOp::ALoad(index),
            OpCode::ILoad(index) => FatOp::ILoad(index),
            OpCode::LLoad(index) => FatOp::LLoad(index),
            OpCode::AStore(index) => FatOp::AStore(index),
            OpCode::DStore(index) => FatOp::DStore(index),
            OpCode::FStore(index) => FatOp::FStore(index),
            OpCode::IStore(index) => FatOp::IStore(index),
            OpCode::LStore(index) => FatOp::LStore(index),
            OpCode::FLoad(index) => FatOp::FLoad(index),
            OpCode::GetStatic(index) => {
                let (ftype, class, name) = fieldref_to_info(index, class);
                match ftype {
                    FieldType::Bool => FatOp::ZGetStatic(class, name),
                    FieldType::Byte => FatOp::BGetStatic(class, name),
                    FieldType::Short => FatOp::SGetStatic(class, name),
                    FieldType::Char => FatOp::CGetStatic(class, name),
                    FieldType::Int => FatOp::IGetStatic(class, name),
                    FieldType::Long => FatOp::LGetStatic(class, name),
                    FieldType::Float => FatOp::FGetStatic(class, name),
                    FieldType::Double => FatOp::DGetStatic(class, name),
                    FieldType::ObjectRef => FatOp::AGetStatic(class, name),
                }
            }
            OpCode::PutStatic(index) => {
                let (ftype, class, name) = fieldref_to_info(index, class);
                match ftype {
                    FieldType::Bool => FatOp::ZPutStatic(class, name),
                    FieldType::Byte => FatOp::BPutStatic(class, name),
                    FieldType::Short => FatOp::SPutStatic(class, name),
                    FieldType::Char => FatOp::CPutStatic(class, name),
                    FieldType::Int => FatOp::IPutStatic(class, name),
                    FieldType::Long => FatOp::LPutStatic(class, name),
                    FieldType::Float => FatOp::FPutStatic(class, name),
                    FieldType::Double => FatOp::DPutStatic(class, name),
                    FieldType::ObjectRef => FatOp::APutStatic(class, name),
                }
            }
            OpCode::GetField(index) => {
                let (ftype, class, name) = fieldref_to_info(index, class);
                match ftype {
                    FieldType::Bool => FatOp::ZGetField(class, name),
                    FieldType::Byte => FatOp::BGetField(class, name),
                    FieldType::Short => FatOp::SGetField(class, name),
                    FieldType::Char => FatOp::CGetField(class, name),
                    FieldType::Int => FatOp::IGetField(class, name),
                    FieldType::Long => FatOp::LGetField(class, name),
                    FieldType::Float => FatOp::FGetField(class, name),
                    FieldType::Double => FatOp::DGetField(class, name),
                    FieldType::ObjectRef => FatOp::AGetField(class, name),
                }
            }
            OpCode::IfICmpEq(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfICmpEq(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfNull(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfNull(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfNotNull(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfNotNull(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfNotZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfICmpNe(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfICmpNe(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfIGreterEqual(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfIGreterEqual(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfGreterEqualZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfGreterEqualZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfGreterZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfGreterZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfLessZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfLessZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfICmpLessEqual(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfICmpLessEqual(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfICmpLessThan(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfICmpLess(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfLessEqualZero(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfLessEqualZero(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::IfACmpNe(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::IfACmpNe(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::GoTo(op_offset) => {
                let op_offset: u16 = (op.1 as i32 + op_offset as i32) as u16;
                FatOp::GoTo(find_op_with_offset(ops, op_offset).unwrap())
            }
            OpCode::PutField(index) => {
                let (ftype, class, name) = fieldref_to_info(index, class);
                match ftype {
                    FieldType::Bool => FatOp::ZPutField(class, name),
                    FieldType::Byte => FatOp::BPutField(class, name),
                    FieldType::Short => FatOp::SPutField(class, name),
                    FieldType::Char => FatOp::CPutField(class, name),
                    FieldType::Int => FatOp::IPutField(class, name),
                    FieldType::Long => FatOp::LPutField(class, name),
                    FieldType::Float => FatOp::FPutField(class, name),
                    FieldType::Double => FatOp::DPutField(class, name),
                    FieldType::ObjectRef => FatOp::APutField(class, name),
                }
            }
            OpCode::New(index) => {
                let class_name = class.lookup_class(index).unwrap();
                FatOp::New(class_name.into())
            }
            OpCode::ANewArray(index) => {
                let class_name = class.lookup_class(index).unwrap();
                FatOp::ANewArray(class_name.into())
            }
            OpCode::CheckCast(index) => {
                let class_name = class.lookup_class(index).unwrap();
                FatOp::CheckedCast(class_name.into())
            }
            OpCode::InstanceOf(index) => {
                let class_name = class.lookup_class(index).unwrap();
                FatOp::InstanceOf(class_name.into())
            }
            OpCode::Dup => FatOp::Dup,
            OpCode::Pop => FatOp::Pop,
            OpCode::Pop2 => FatOp::Pop2,
            ///TODO: handle non-static methods(change argc by 1)
            OpCode::InvokeSpecial(index) => {
                let (name, mut argc) = methodref_to_mangled_and_argc(index, class);
                // Either <init> or <cinit>
                if name.contains('<') {
                    argc += 1;
                }
                FatOp::InvokeSpecial(name, argc)
            }
            OpCode::InvokeStatic(index) => {
                let (name, argc) = methodref_to_mangled_and_argc(index, class);
                FatOp::InvokeStatic(name, argc)
            }
            OpCode::InvokeVirtual(index) => {
                let (class, name, argc) = methodref_to_partial_mangled_and_argc(index, class);
                FatOp::InvokeVirtual(class, name, argc + 1)
            }
            OpCode::InvokeInterface(index) => {
                let (name, argc) = methodref_to_mangled_and_argc(index, class);
                //TODO:Potentially handle static interface methods.
                FatOp::InvokeInterface(name, argc + 1)
            }
            OpCode::InvokeDynamic(index) => {
                let (bootstrap_method_attr_index, name_and_type_index) =
                    class.lookup_invoke_dynamic(index).unwrap();
                let bootstrap_method = class
                    .lookup_bootstrap_method(bootstrap_method_attr_index)
                    .unwrap();
                let (reference_kind, reference_index) = class
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
            OpCode::AALoad => FatOp::AALoad,
            OpCode::BALoad => FatOp::BALoad,
            OpCode::CALoad => FatOp::CALoad,
            OpCode::DALoad => FatOp::DALoad,
            OpCode::FALoad => FatOp::FALoad,
            OpCode::IALoad => FatOp::IALoad,
            OpCode::LALoad => FatOp::LALoad,
            OpCode::ArrayLength => FatOp::ArrayLength,
            OpCode::IInc(local, offset) => FatOp::IInc(local, offset),
            OpCode::Throw => FatOp::Throw,
            _ => todo!("can't expand op {op:?}"),
        };
        fatops.push(cop);
    }
    fatops.into()
}

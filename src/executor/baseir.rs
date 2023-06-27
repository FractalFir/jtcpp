use super::ExecCtx;
use super::UnmetDependency;
use crate::executor::fatops::FatOp;
use crate::ClassRef;
use crate::ExecEnv;
use crate::{ExecException, Value};
#[derive(Debug)]
pub(crate) enum BaseIR {
    Dup,
    DupX1,
    Pop,
    Swap,
    AConst(usize),
    DConst(f64),
    FConst(f32),
    IConst(i32),
    LConst(i64),
    ALoad(u8),
    FLoad(u8),
    DLoad(u8),
    ILoad(u8),
    LLoad(u8),
    AStore(u8),
    DStore(u8),
    FStore(u8),
    IStore(u8),
    LStore(u8),
    IAnd,
    LAnd,
    DAdd,
    FAdd,
    IAdd,
    LAdd,
    DSub,
    FSub,
    ISub,
    LSub,
    IMul,
    LMul,
    FMul,
    DMul,
    DRem,
    FRem,
    IRem,
    LRem,
    DDiv,
    FDiv,
    IDiv,
    LDiv,
    INeg,
    LNeg,
    DNeg,
    FNeg,
    LCmp,
    DCmpL,
    DCmpG,
    FCmpL,
    FCmpG,
    IOr,
    LOr,
    IXOr,
    LXOr,
    LUShr,
    IUShr,
    InvokeSpecial(usize, u8),
    InvokeStatic(usize, u8),
    InvokeVirtual(usize, u8),
    ZGetStatic(usize),
    BGetStatic(usize),
    SGetStatic(usize),
    IGetStatic(usize),
    LGetStatic(usize),
    FGetStatic(usize),
    DGetStatic(usize),
    AGetStatic(usize),
    CGetStatic(usize),
    ZPutStatic(usize),
    BPutStatic(usize),
    SPutStatic(usize),
    IPutStatic(usize),
    LPutStatic(usize),
    FPutStatic(usize),
    DPutStatic(usize),
    APutStatic(usize),
    CPutStatic(usize),
    ZGetField(usize),
    BGetField(usize),
    SGetField(usize),
    IGetField(usize),
    LGetField(usize),
    FGetField(usize),
    DGetField(usize),
    AGetField(usize),
    CGetField(usize),
    ZPutField(usize),
    BPutField(usize),
    SPutField(usize),
    IPutField(usize),
    LPutField(usize),
    FPutField(usize),
    DPutField(usize),
    APutField(usize),
    CPutField(usize),
    Return,
    AReturn,
    FReturn,
    DReturn,
    IReturn,
    LReturn,
    IShr,
    IShl,
    LShr,
    LShl,
    F2D,
    D2F,
    I2L,
    F2L,
    L2I,
    I2F,
    L2F,
    F2I,
    I2B,
    I2C,
    I2S,
    I2D,
    D2I,
    D2L,
    L2D,
    New(ClassRef),
    CheckedCast(ClassRef),
    InstanceOf(ClassRef),
    ANewArray(ClassRef),
    MultiANewArray(ClassRef,u8),
    MultiBNewArray(u8),
    MultiCNewArray(u8),
    MultiDNewArray(u8),
    MultiFNewArray(u8),
    MultiINewArray(u8),
    MultiLNewArray(u8),
    MultiSNewArray(u8),
    MultiZNewArray(u8),
    BNewArray,
    CNewArray,
    DNewArray,
    FNewArray,
    INewArray,
    LNewArray,
    SNewArray,
    ZNewArray,
    IfIGreterEqual(usize),
    IfGreterEqualZero(usize),
    IfLessZero(usize),
    IfICmpLessEqual(usize),
    IfICmpLess(usize),
    IfLessEqualZero(usize),
    IfICmpNe(usize),
    IfICmpEq(usize),
    IfNull(usize),
    IfNotNull(usize),
    IfACmpNe(usize),
    IfACmpEq(usize),
    IfZero(usize),
    IfICmpGreater(usize),
    IfGreterZero(usize),
    GoTo(usize),
    IInc(u8, i8),
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
    FALoad,
    DALoad,
    IALoad,
    LALoad,
    SALoad,
    ZALoad,
    ArrayLength,
    Throw,
    MonitorEnter,
    MonitorExit,
    Invalid, //Special op which represents invalid op which should have not been produced.
}
fn lookup_static(
    class_name: &str,
    field_name: &str,
    exec_env: &mut ExecEnv,
) -> Result<usize, UnmetDependency> {
    let class_id = exec_env.code_container.lookup_class(class_name);
    let class_id = if let Some(class_id) = class_id {
        class_id
    } else {
        return Err(UnmetDependency::NeedsClass(class_name.into()));
    };
    if let Some(static_id) = exec_env.code_container.classes[class_id].get_static(field_name) {
        Ok(static_id)
    } else {
        println!("{class_name} does not have static field {field_name}! Skipping this method to try and run anyway!");
        Err(UnmetDependency::MissingField(class_name.into(),field_name.into()))
    }
}
fn lookup_field(
    class_name: &str,
    field_name: &str,
    exec_env: &mut ExecEnv,
) -> Result<usize, UnmetDependency> {
    let class_id = exec_env.code_container.lookup_class(class_name);
    let class_id = if let Some(class_id) = class_id {
        class_id
    } else {
        return Err(UnmetDependency::NeedsClass(class_name.into()));
    };
    if let Some((field_id, _ftype)) =
        exec_env.code_container.classes[class_id].get_field(field_name)
    {
        Ok(field_id)
    } else {
        panic!("Can't find field {field_name} on {class_name}!");
    }
}
pub(crate) fn into_base(
    fat: &[FatOp],
    exec_env: &mut ExecEnv,
) -> Result<Box<[BaseIR]>, UnmetDependency> {
    //println!("Fat:{fat:?}");
    let mut newops = Vec::with_capacity(fat.len());
    for op in fat {
        let newop = match op {
            FatOp::AConstNull => BaseIR::AConst(0),
            FatOp::BConst(byte) => BaseIR::IConst(*byte as i32),
            FatOp::DConst(double) => BaseIR::DConst(*double as f64),
            FatOp::FConst(float) => BaseIR::FConst(*float as f32),
            FatOp::SConst(short) => BaseIR::IConst(*short as i32),
            FatOp::LConst(long) => BaseIR::LConst(*long as i64),
            FatOp::IInc(local, change) => BaseIR::IInc(*local, *change),
            FatOp::IfIGreterEqual(index) => BaseIR::IfIGreterEqual(*index),
            FatOp::IfICmpNe(index) => BaseIR::IfICmpNe(*index),
            FatOp::IfGreterEqualZero(index) => BaseIR::IfGreterEqualZero(*index),
            FatOp::IfLessEqualZero(index) => BaseIR::IfLessEqualZero(*index),
            FatOp::IfACmpNe(index) => BaseIR::IfACmpNe(*index),
            FatOp::IfNull(index) => BaseIR::IfNull(*index),
            FatOp::IfNotNull(index) => BaseIR::IfNotNull(*index),
            FatOp::IfZero(index) => BaseIR::IfZero(*index),
            FatOp::IfICmpEq(index) => BaseIR::IfICmpEq(*index),
            FatOp::IfLessZero(index) => BaseIR::IfLessZero(*index),
            FatOp::IfICmpLess(index) => BaseIR::IfICmpLess(*index),
            FatOp::IfICmpGreater(index) => BaseIR::IfICmpGreater(*index),
            FatOp::IfICmpLessEqual(index) => BaseIR::IfICmpLessEqual(*index),
            FatOp::IfGreterZero(index) => BaseIR::IfGreterZero(*index),
            FatOp::IfACmpEq(index) => BaseIR::IfACmpEq(*index),
            FatOp::GoTo(index) => BaseIR::GoTo(*index),
            FatOp::FConst(float) => BaseIR::FConst(*float),
            FatOp::IConst(int) => BaseIR::IConst(*int),
            FatOp::IAnd => BaseIR::IAnd,
            FatOp::LAnd => BaseIR::LAnd,
            FatOp::DSub => BaseIR::DSub,
            FatOp::FSub => BaseIR::FSub,
            FatOp::ISub => BaseIR::ISub,
            FatOp::LSub => BaseIR::LSub,
            FatOp::DAdd => BaseIR::DAdd,
            FatOp::FAdd => BaseIR::FAdd,
            FatOp::IAdd => BaseIR::IAdd,
            FatOp::LAdd => BaseIR::LAdd,
            FatOp::IMul => BaseIR::IMul,
            FatOp::LMul => BaseIR::LMul,
            FatOp::DMul => BaseIR::DMul,
            FatOp::FMul => BaseIR::FMul,
            FatOp::DDiv => BaseIR::DDiv,
            FatOp::FDiv => BaseIR::FDiv,
            FatOp::IDiv => BaseIR::IDiv,
            FatOp::LDiv => BaseIR::LDiv,
            FatOp::IRem => BaseIR::IRem,
            FatOp::LRem => BaseIR::LRem,
            FatOp::DRem => BaseIR::DRem,
            FatOp::FRem => BaseIR::FRem,
            FatOp::INeg => BaseIR::INeg,
            FatOp::LNeg => BaseIR::LNeg,
            FatOp::DNeg => BaseIR::DNeg,
            FatOp::FNeg => BaseIR::FNeg,
            FatOp::IShr => BaseIR::IShr,
            FatOp::IShl => BaseIR::IShl,
            FatOp::LShr => BaseIR::LShr,
            FatOp::LShl => BaseIR::LShl,
            FatOp::F2D => BaseIR::F2D,
            FatOp::D2F => BaseIR::D2F,
            FatOp::L2I => BaseIR::L2I,
            FatOp::F2L => BaseIR::F2L,
            FatOp::I2F => BaseIR::I2F,
            FatOp::F2I => BaseIR::F2I,
            FatOp::I2B => BaseIR::I2B,
            FatOp::I2C => BaseIR::I2C,
            FatOp::I2D => BaseIR::I2D,
            FatOp::L2F => BaseIR::L2F,
            FatOp::I2S => BaseIR::I2S,
            FatOp::I2L => BaseIR::I2L,
            FatOp::D2I => BaseIR::D2I,
            FatOp::D2L => BaseIR::D2L,
            FatOp::L2D => BaseIR::L2D,
            FatOp::LCmp => BaseIR::LCmp,
            FatOp::DCmpL => BaseIR::DCmpL,
            FatOp::DCmpG => BaseIR::DCmpG,
            FatOp::FCmpL => BaseIR::FCmpL,
            FatOp::FCmpG => BaseIR::FCmpG,
            FatOp::IOr => BaseIR::IOr,
            FatOp::LOr => BaseIR::LOr,
            FatOp::IXOr => BaseIR::IXOr,
            FatOp::LXOr => BaseIR::LXOr,
            FatOp::LUShr => BaseIR::LUShr,
            FatOp::IUShr => BaseIR::IUShr,
            FatOp::ALoad(index) => BaseIR::ALoad(*index),
            FatOp::ILoad(index) => BaseIR::ILoad(*index),
            FatOp::AStore(index) => BaseIR::AStore(*index),
            FatOp::FStore(index) => BaseIR::FStore(*index),
            FatOp::DStore(index) => BaseIR::DStore(*index),
            FatOp::IStore(index) => BaseIR::IStore(*index),
            FatOp::LStore(index) => BaseIR::LStore(*index),
            FatOp::FLoad(index) => BaseIR::FLoad(*index),
            FatOp::DLoad(index) => BaseIR::DLoad(*index),
            FatOp::LLoad(index) => BaseIR::LLoad(*index),
            FatOp::InvokeSpecial(mangled, args) => {
                let method_id = exec_env.code_container.lookup_or_insert_method(&mangled);
                //println!("special mangled:{mangled} id:{method_id}");
                BaseIR::InvokeSpecial(method_id, *args)
            }
            FatOp::InvokeStatic(mangled, args) => {
                let method_id = exec_env.code_container.lookup_or_insert_method(&mangled);
                BaseIR::InvokeStatic(method_id, *args)
            }
            FatOp::InvokeVirtual(class_name, method, argc) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let virtual_index = exec_env.code_container.classes[class_id]
                    .lookup_virtual(method)
                    .unwrap();
                BaseIR::InvokeVirtual(virtual_index, *argc)
            }
            FatOp::AGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::AGetField(field_id)
            }
            FatOp::BGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::BGetField(field_id)
            }
            FatOp::CGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::CGetField(field_id)
            }
            FatOp::DGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::DGetField(field_id)
            }
            FatOp::FGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::FGetField(field_id)
            }
            FatOp::IGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::IGetField(field_id)
            }
            FatOp::SGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::SGetField(field_id)
            }
            FatOp::LGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::LGetField(field_id)
            }
            FatOp::ZGetField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::ZGetField(field_id)
            }
            FatOp::APutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::APutField(field_id)
            }
            FatOp::BPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::BPutField(field_id)
            }
            FatOp::CPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::CPutField(field_id)
            }
            FatOp::DPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::DPutField(field_id)
            }
            FatOp::FPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::FPutField(field_id)
            }
            FatOp::LPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::LPutField(field_id)
            }
            FatOp::IPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::IPutField(field_id)
            }
            FatOp::LPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::LPutField(field_id)
            }
            FatOp::SPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::SPutField(field_id)
            }
            FatOp::ZPutField(class_name, field_name) => {
                let field_id = lookup_field(class_name, field_name, exec_env)?;
                BaseIR::ZPutField(field_id)
            }
            FatOp::AGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::AGetStatic(static_id)
            }
            FatOp::BGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::BGetStatic(static_id)
            }
            FatOp::CGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::CGetStatic(static_id)
            }
            FatOp::DGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::DGetStatic(static_id)
            }
            FatOp::FGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::FGetStatic(static_id)
            }
            FatOp::IGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::IGetStatic(static_id)
            }
            FatOp::LGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::LGetStatic(static_id)
            }
            FatOp::SGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::SGetStatic(static_id)
            }
            FatOp::ZGetStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::ZGetStatic(static_id)
            }
            FatOp::APutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::APutStatic(static_id)
            }
            FatOp::BPutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::BPutStatic(static_id)
            }
            FatOp::CPutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::CPutStatic(static_id)
            }
            FatOp::DPutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::DPutStatic(static_id)
            }
            FatOp::FPutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::FPutStatic(static_id)
            }
            FatOp::IPutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::IPutStatic(static_id)
            }
            FatOp::LPutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::LPutStatic(static_id)
            }
            FatOp::ZPutStatic(class_name, field_name) => {
                let static_id = lookup_static(class_name, field_name, exec_env)?;
                BaseIR::ZPutStatic(static_id)
            }
            FatOp::MultiANewArray(type_name,dimensions) => {
                use super::FieldType;
                use crate::executor::field_desc_str_to_ftype;
                let ftype = field_desc_str_to_ftype(type_name,*dimensions as usize);
                match ftype{
                    FieldType::ObjectRef => {
                        let class_name = &type_name[(*dimensions as usize + 1)..(type_name.len() - 1)];
                        let class_id = exec_env.code_container.lookup_class(class_name);
                        let class_id = if let Some(class_id) = class_id {
                            class_id
                        } else {
                            return Err(UnmetDependency::NeedsClass(class_name.into()));
                        };
                        BaseIR::MultiANewArray(class_id,*dimensions)
                    },
                    FieldType::Byte => BaseIR::MultiBNewArray(*dimensions),
                    FieldType::Char => BaseIR::MultiCNewArray(*dimensions),
                    FieldType::Double => BaseIR::MultiDNewArray(*dimensions),
                    FieldType::Float => BaseIR::MultiFNewArray(*dimensions),
                    FieldType::Int => BaseIR::MultiINewArray(*dimensions),
                    FieldType::Short => BaseIR::MultiSNewArray(*dimensions),
                    FieldType::Long => BaseIR::MultiLNewArray(*dimensions),
                    _=> todo!("unhandled multi array type {ftype:?}"),
                }
            }
            FatOp::New(class_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                BaseIR::New(class_id)
            }
            FatOp::CheckedCast(class_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                BaseIR::CheckedCast(class_id)
            }
            FatOp::InstanceOf(class_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                BaseIR::InstanceOf(class_id)
            }
            FatOp::ANewArray(class_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                BaseIR::ANewArray(class_id)
            }
            FatOp::BNewArray => BaseIR::BNewArray,
            FatOp::CNewArray => BaseIR::CNewArray,
            FatOp::DNewArray => BaseIR::DNewArray,
            FatOp::FNewArray => BaseIR::FNewArray,
            FatOp::INewArray => BaseIR::INewArray,
            FatOp::LNewArray => BaseIR::LNewArray,
            FatOp::SNewArray => BaseIR::SNewArray,
            FatOp::ZNewArray => BaseIR::ZNewArray,
            FatOp::StringConst(string) => BaseIR::AConst(exec_env.const_string(string)),
            FatOp::ClassConst(class_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                BaseIR::AConst(exec_env.const_class(class_id))
            }
            FatOp::Return => BaseIR::Return,
            FatOp::AReturn => BaseIR::AReturn,
            FatOp::IReturn => BaseIR::IReturn,
            FatOp::DReturn => BaseIR::DReturn,
            FatOp::FReturn => BaseIR::FReturn,
            FatOp::LReturn => BaseIR::LReturn,
            FatOp::Swap => BaseIR::Swap,
            FatOp::Dup => BaseIR::Dup,
            FatOp::DupX1 => BaseIR::DupX1,
            FatOp::Pop => BaseIR::Pop,
            FatOp::AAStore => BaseIR::AAStore,
            FatOp::BAStore => BaseIR::BAStore,
            FatOp::CAStore => BaseIR::CAStore,
            FatOp::DAStore => BaseIR::DAStore,
            FatOp::FAStore => BaseIR::FAStore,
            FatOp::IAStore => BaseIR::IAStore,
            FatOp::LAStore => BaseIR::LAStore,
            FatOp::SAStore => BaseIR::SAStore,
            FatOp::ZAStore => BaseIR::ZAStore,
            FatOp::AALoad => BaseIR::AALoad,
            FatOp::BALoad => BaseIR::BALoad,
            FatOp::CALoad => BaseIR::CALoad,
            FatOp::FALoad => BaseIR::FALoad,
            FatOp::DALoad => BaseIR::DALoad,
            FatOp::IALoad => BaseIR::IALoad,
            FatOp::LALoad => BaseIR::LALoad,
            FatOp::SALoad => BaseIR::SALoad,
            FatOp::ZALoad => BaseIR::ZALoad,
            FatOp::ArrayLength => BaseIR::ArrayLength,
            FatOp::MonitorEnter => BaseIR::MonitorEnter,
            FatOp::MonitorExit => BaseIR::MonitorExit,
            FatOp::Throw => BaseIR::Throw,
            //TEMPORARY!
            FatOp::InvokeDynamic => BaseIR::Invalid,
            FatOp::InvokeInterface(_, _) => BaseIR::Invalid,
            // Funky behaviour with doubles and longs.
            FatOp::Pop2 => BaseIR::Invalid,
            FatOp::Dup2 => BaseIR::Invalid,
            FatOp::DupX2 => BaseIR::Invalid,
            FatOp::Dup2X2 => BaseIR::Invalid,
            FatOp::Dup2X1 => BaseIR::Invalid,
            _ => todo!("Can't convert op {op:?} to base IR"),
        };
        //println!("Op:{op:?} new_op:{newop:?}");
        newops.push(newop);
    }
    Ok(newops.into())
}
pub(crate) fn call<'caller, 'env>(ops: &[BaseIR], mut ctx: ExecCtx) -> Result<Value, ExecException>
where
    'caller: 'env,
{
    let mut op_index = 0;
    //let mut stack: Vec<Value> = Vec::with_capacity(100);
    //let mut locals: Vec<_> = args.into();
    //TODO: get actual number!
    //let max_locals = 100;
    //while locals.len() < max_locals as usize {
    //locals.push(Value::Void);
    //}
    loop {
        let op = &ops[op_index];
        println!("op:{op:?}");
        match op {
            BaseIR::New(class_ref) => {
                let new_obj = ctx.new_obj(*class_ref);
                ctx.stack_push(Value::ObjectRef(new_obj));
            }
            BaseIR::ANewArray(_) => {
                let length = ctx.stack_pop().unwrap().as_int().unwrap() as usize;
                //let new_obj = ctx.new_obj(*class_ref);
                let new_arr = ctx.new_array(Value::ObjectRef(0), length);
                ctx.stack_push(Value::ObjectRef(new_arr));
            }
            BaseIR::ArrayLength => {
                let arr = ctx.stack_pop().unwrap().as_objref().unwrap() as usize;
                ctx.stack_push(Value::Int(ctx.get_array_length(arr) as i32));
            }
            BaseIR::Dup => {
                let a: Value = ctx.stack_pop().unwrap().clone();
                ctx.stack_push(a);
                ctx.stack_push(a);
            }
            BaseIR::IConst(value) => ctx.stack_push(Value::Int(*value)),
            BaseIR::AConst(value) => ctx.stack_push(Value::ObjectRef(*value)),
            BaseIR::ILoad(index) => ctx.stack_push(ctx.get_local(*index).unwrap().clone()),
            BaseIR::FLoad(index) => ctx.stack_push(ctx.get_local(*index).unwrap().clone()),
            BaseIR::ALoad(index) => {
                let local = ctx.get_local(*index).unwrap().clone();
                assert_ne!(local, Value::Void, "Loading local at {index} yelded Void!");
                ctx.stack_push(local)
            }
            BaseIR::IStore(index) => {
                let a = ctx.stack_pop().unwrap();
                ctx.set_local(*index, a.clone());
            }
            BaseIR::FStore(index) => {
                let a = ctx.stack_pop().unwrap();
                ctx.set_local(*index, a.clone());
            }
            BaseIR::IAdd => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a + b));
            }
            BaseIR::FAdd => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a + b));
            }
            BaseIR::ISub => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a - b));
            }
            BaseIR::FSub => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a - b));
            }
            BaseIR::IMul => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a * b));
            }
            BaseIR::FMul => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a * b));
            }
            BaseIR::IDiv => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a / b));
            }
            BaseIR::FDiv => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a / b));
            }
            BaseIR::IRem => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a % b));
            }
            BaseIR::APutField(id) => {
                let val = ctx.stack_pop().unwrap();
                let obj_ref = ctx.stack_pop().unwrap();
                let obj_ref = obj_ref
                    .as_objref()
                    .expect(&format!("Expected object reference, got {obj_ref:?}!"));
                ctx.put_field(obj_ref, *id, val);
            }
            BaseIR::FPutField(id) => {
                let val = ctx.stack_pop().unwrap();
                let obj_ref = ctx.stack_pop().unwrap();
                let obj_ref = obj_ref.as_objref().unwrap();
                ctx.put_field(obj_ref, *id, val);
            }
            BaseIR::FGetField(id) => {
                let obj_ref = ctx.stack_pop().unwrap().as_objref().unwrap();
                let value = ctx.get_field(obj_ref, *id).unwrap();
                ctx.stack_push(value);
            }
            BaseIR::AGetField(id) => {
                let obj_ref = ctx.stack_pop().unwrap().as_objref().unwrap();
                let value = ctx.get_field(obj_ref, *id).unwrap();
                ctx.stack_push(value);
            }
            BaseIR::AGetStatic(index) => {
                ctx.stack_push(ctx.get_static(*index));
            }
            BaseIR::IReturn | BaseIR::FReturn | BaseIR::AReturn => {
                return Ok(ctx.stack_pop().unwrap());
            }
            BaseIR::Return => {
                return Ok(Value::Void);
            }
            BaseIR::InvokeStatic(method_id, argc) => {
                let mut args: Box<[Value]> = (0..*argc).map(|_| ctx.stack_pop().unwrap()).collect();
                args.reverse();
                // Hack
                let args = args;
                let res: Value = ctx.invoke_method(&args, *method_id)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }
            BaseIR::InvokeSpecial(method_id, argc) => {
                println!("method_id:{method_id}");
                let mut args: Box<[Value]> = (0..*argc).map(|_| ctx.stack_pop().unwrap()).collect();
                args.reverse();
                // Hack
                let args = args;
                let res: Value = ctx.invoke_method(&args, *method_id)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }
            BaseIR::InvokeVirtual(method_id, argc) => {
                let mut args: Box<[Value]> = (0..*argc).map(|_| ctx.stack_pop().unwrap()).collect();
                args.reverse();
                let obj_ref = args[0].as_objref().unwrap();
                let virtual_method = ctx.get_virtual(obj_ref, *method_id).unwrap();
                //todo!("virtual:{virtual_method:?}");
                let res: Value = ctx.invoke_method(&args, virtual_method)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }
            BaseIR::IfIGreterEqual(jump_index) => {
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                if a >= b {
                    op_index = *jump_index;
                    continue;
                }
            }
            _ => todo!("Can't execute {op:?} yet!"),
        }
        op_index += 1;
    }
}

use super::{load_i16, load_i8, load_u16, load_u8,load_i32};
#[derive(Debug, Clone)]
pub(crate) enum OpCode {
    Nop,
    ALoad(u8),
    FLoad(u8),
    DLoad(u8),
    ILoad(u8),
    LLoad(u8),
    DConst(f64),
    FConst(f32),
    IConst(i32),
    LConst(i64),
    AConstNull,
    AStore(u8),
    DStore(u8),
    FStore(u8),
    IStore(u8),
    LStore(u8),
    LAdd,
    IAdd,
    DAdd,
    FAdd,
    LSub,
    ISub,
    DSub,
    FSub,
    LMul,
    IMul,
    DMul,
    FMul,
    DDiv,
    FDiv,
    IDiv,
    LDiv,
    FRem,
    DRem,
    IRem,
    LRem,
    IShr,
    IShl,
    LShr,
    LShl,
    IUShr,
    LUShr,
    LUShl,
    IAnd,
    LAnd,
    IOr,
    LOr,
    IXOr,
    LXOr,
    DNeg,
    FNeg,
    INeg,
    LNeg,
    IInc(u8, i8),
    InvokeSpecial(u16),
    InvokeVirtual(u16),
    InvokeInterface(u16),
    InvokeStatic(u16),
    InvokeDynamic(u16),
    Return,
    AReturn,
    IReturn,
    DReturn,
    FReturn,
    LReturn,
    GetStatic(u16),
    PutStatic(u16),
    GetField(u16),
    PutField(u16),
    LoadConst(u16),
    IfICmpEq(i16),
    IfICmpNe(i16),
    IfICmpLessEqual(i16),
    IfICmpLessThan(i16),
    IfICmpGreater(i16),
    IfZero(i16),    //aka IfEq on wikipedia
    IfNotZero(i16), //aka IfEq on wikipedia
    IfNull(i16),
    IfNotNull(i16),
    IfACmpNe(i16),
    IfACmpEq(i16),
    IfIGreterEqual(i16),
    IfGreterEqualZero(i16), // aka IfGe
    IfGreterZero(i16),      // aka IfGt
    IfLessZero(i16),        // aka IfLt
    IfLessEqualZero(i16),   // aka IfLt
    GoTo(i16),
    Dup,
    DupX1,
    Dup2X1,
    Dup2X2,
    DupX2,
    Dup2,
    Swap,
    Pop,
    Pop2,
    New(u16),
    NewArray(u8),
    ANewArray(u16),
    MultiANewArray(u16, u8),
    BIPush(i8),
    SIPush(i16),
    ArrayLength,
    Throw,
    AALoad,
    BALoad,
    CALoad,
    FALoad,
    DALoad,
    IALoad,
    LALoad,
    SALoad,
    AAStore,
    BAStore,
    CAStore,
    DAStore,
    FAStore,
    IAStore,
    LAStore,
    SAStore,
    CheckCast(u16),
    InstanceOf(u16), // Check if obj is an instance of class and then???
    D2F,
    D2I,
    D2L,
    F2I,
    F2L,
    F2D,
    I2B,
    I2C,
    I2D,
    I2F,
    I2L,
    I2S,
    L2I,
    L2F,
    L2D,
    LCmp,  //Compare two longs. Push 0 if same, 1 if a > b, -1 if b > a
    FCmpL, //Compare two floats. Push 0 if same, 1 if a > b, -1 if b > a, and -1 if a or b is NaN.
    FCmpG, //Compare two floats. Push 0 if same, 1 if a > b, -1 if b > a, and 1 if a or b is NaN.
    DCmpL, //Compare two doubles. Push 0 if same, 1 if a > b, -1 if b > a, and -1 if a or b is NaN.
    DCmpG, //Compare two doubles. Push 0 if same, 1 if a > b, -1 if b > a, and 1 if a or b is NaN.
    MonitorEnter,
    MonitorExit,
    Reserved, // Should never appear, sign of error.
    LookupSwitch(Box<LookupSwitch>),
}
///Separate to decrease footprint of individual OP.
 #[derive(Debug,Clone)]
pub(crate) struct LookupSwitch{
    pub(crate) default_offset:i32,
    pub(crate) pairs:Box<[(i32,i32)]>,
}
impl OpCode {
    //Checks if op is a valid method terminator(return or throw, or GoTo that goes back).
    fn is_term(&self) -> bool {
        matches!(
            self,
            Self::Return
                | Self::AReturn
                | Self::IReturn
                | Self::FReturn
                | Self::LReturn
                | Self::DReturn
                | Self::Throw
                | Self::GoTo(..=-1)
        )
    }
}
pub(crate) fn load_ops<R: std::io::Read>(
    src: &mut R,
    code_length: u32,
) -> Result<Vec<(OpCode, u16)>, std::io::Error> {
    let mut curr_offset:u16 = 0;

    let mut ops = Vec::with_capacity(code_length as usize);
    //println!("\nMethod begin\n");
    while (curr_offset as u32) < code_length {
        let op = load_u8(src)?;
        let op_offset = curr_offset;
        //print!("{curr_offset}:\t");
        curr_offset += 1;
        let decoded_op = match op {
            0x0 => OpCode::Nop,
            0x1 => OpCode::AConstNull,
            0x2..=0x8 => OpCode::IConst(op as i32 - 0x3),
            0x9..=0xa => OpCode::LConst(op as i64 - 0x9),
            0xb => OpCode::FConst(0.0),
            0xc => OpCode::FConst(1.0),
            0xd => OpCode::FConst(2.0),
            0xe => OpCode::DConst(0.0),
            0xf => OpCode::DConst(1.0),
            0x10 => {
                let value = load_i8(src)?;
                curr_offset += 1;
                OpCode::BIPush(value)
            }
            0x11 => {
                let value = load_i16(src)?;
                curr_offset += 2;
                OpCode::SIPush(value)
            }
            0x12 => {
                let constant_pool_index = load_u8(src)?;
                curr_offset += 1;
                OpCode::LoadConst(constant_pool_index as u16)
            }
            0x13 | 0x14 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::LoadConst(constant_pool_index)
            }
            0x15 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::ILoad(index)
            }
            0x16 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::LLoad(index)
            }
            0x17 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::FLoad(index)
            }
            0x18 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::DLoad(index)
            }
            0x19 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::ALoad(index)
            }
            0x1a..=0x1d => OpCode::ILoad(op - 0x1a),
            0x1e..=0x21 => OpCode::LLoad(op - 0x1e),
            0x22..=0x25 => OpCode::FLoad(op - 0x22),
            0x26..=0x29 => OpCode::DLoad(op - 0x26),
            0x2a..=0x2d => OpCode::ALoad(op - 0x2a),
            0x2e => OpCode::IALoad,
            0x2f => OpCode::LALoad,
            0x30 => OpCode::FALoad,
            0x31 => OpCode::DALoad,
            0x32 => OpCode::AALoad,
            0x33 => OpCode::BALoad,
            0x34 => OpCode::CALoad,
            0x35 => OpCode::SALoad,
            0x3a => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::IStore(index)
            }
            0x37 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::LStore(index)
            }
            0x38 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::FStore(index)
            }
            0x39 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::DStore(index)
            }
            0x3b..=0x3e => OpCode::IStore(op - 0x3b),
            0x3f..=0x42 => OpCode::LStore(op - 0x3f),
            0x43..=0x46 => OpCode::FStore(op - 0x43),
            0x47..=0x4a => OpCode::DStore(op - 0x47),
            0x4b..=0x4e => OpCode::AStore(op - 0x4b),
            0x36 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::IStore(index)
            }
            0x4f => OpCode::IAStore,
            0x50 => OpCode::LAStore,
            0x51 => OpCode::FAStore,
            0x52 => OpCode::DAStore,
            0x53 => OpCode::AAStore,
            0x54 => OpCode::BAStore,
            0x55 => OpCode::CAStore,
            0x56 => OpCode::SAStore,
            0x57 => OpCode::Pop,
            0x58 => OpCode::Pop2,
            0x59 => OpCode::Dup,
            0x5a => OpCode::DupX1,
            0x5b => OpCode::DupX2,
            0x5c => OpCode::Dup2,
            0x5d => OpCode::Dup2X1,
            0x5e => OpCode::Dup2X2,
            0x5f => OpCode::Swap,
            0x60 => OpCode::IAdd,
            0x61 => OpCode::LAdd,
            0x62 => OpCode::FAdd,
            0x63 => OpCode::DAdd,
            0x64 => OpCode::ISub,
            0x65 => OpCode::LSub,
            0x66 => OpCode::FSub,
            0x67 => OpCode::DSub,
            0x68 => OpCode::IMul,
            0x69 => OpCode::LMul,
            0x6a => OpCode::FMul,
            0x6b => OpCode::DMul,
            0x6c => OpCode::IDiv,
            0x6d => OpCode::LDiv,
            0x6e => OpCode::FDiv,
            0x6f => OpCode::DDiv,
            0x70 => OpCode::IRem,
            0x71 => OpCode::LRem,
            0x72 => OpCode::FRem,
            0x73 => OpCode::DRem,
            0x74 => OpCode::INeg,
            0x75 => OpCode::LNeg,
            0x76 => OpCode::FNeg,
            0x77 => OpCode::DNeg,
            0x78 => OpCode::IShl,
            0x79 => OpCode::LShl,
            0x7a => OpCode::IShr,
            0x7b => OpCode::IUShr,
            0x7c => OpCode::LUShr,
            0x7d => OpCode::LUShr,
            0x7e => OpCode::IAnd,
            0x7f => OpCode::LAnd,
            0x80 => OpCode::IOr,
            0x81 => OpCode::LOr,
            0x82 => OpCode::IXOr,
            0x83 => OpCode::LXOr,
            0x84 => {
                let var = load_u8(src)?;
                let incr = load_i8(src)?;
                curr_offset += 2;
                OpCode::IInc(var, incr)
            }
            0x85 => OpCode::I2L,
            0x86 => OpCode::I2F,
            0x87 => OpCode::I2D,
            0x88 => OpCode::L2I,
            0x89 => OpCode::L2F,
            0x8a => OpCode::L2D,
            0x8b => OpCode::F2I,
            0x8c => OpCode::F2L,
            0x8d => OpCode::F2D,
            0x8e => OpCode::D2I,
            0x8f => OpCode::D2L,
            0x90 => OpCode::D2F,
            0x91 => OpCode::I2B,
            0x92 => OpCode::I2C,
            0x93 => OpCode::I2S,
            0x94 => OpCode::LCmp,
            0x95 => OpCode::FCmpL,
            0x96 => OpCode::FCmpG,
            0x97 => OpCode::DCmpL,
            0x98 => OpCode::DCmpG,
            0x99 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfZero(offset)
            }
            0x9a => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfNotZero(offset)
            }
            0x9b => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfLessZero(offset)
            }
            0x9c => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfGreterEqualZero(offset)
            }
            0x9d => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfGreterZero(offset)
            }
            0x9e => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfLessEqualZero(offset)
            }
            0x9f => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfICmpEq(offset)
            }
            0xa0 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfICmpNe(offset)
            }
            0xa1 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfICmpLessThan(offset)
            }
            0xa2 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfIGreterEqual(offset)
            }
            0xa3 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfICmpGreater(offset)
            }
            0xa4 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfICmpLessEqual(offset)
            }
            0xa5 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfACmpEq(offset)
            }
            0xa6 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfACmpNe(offset)
            }
            0xa7 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::GoTo(offset)
            }
            0xab => {
                let to_next = ((4 - curr_offset % 4)%4) as usize;
                /// skip to_next
                let mut out = [0;4];
                src.read_exact(&mut out[..to_next])?;
                curr_offset += to_next as u16;
                assert_eq!(curr_offset % 4, 0);
                let default_offset = load_i32(src)?;
                curr_offset += 4;
                let npairs = load_i32(src)?;
                curr_offset += 4;
                let mut pairs = Vec::with_capacity(npairs as usize);
                for _ in 0..npairs{
                    let value_match = load_i32(src)?;
                    curr_offset += 4;
                    let offset = load_i32(src)?;
                    curr_offset += 4;
                    pairs.push((value_match,offset));
                }
                OpCode::LookupSwitch(Box::new(LookupSwitch{
                    default_offset,
                    pairs:pairs.into(),
                }))
            }
            0xaa => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Table switch op not supported!",
                ));
            }
            0xac => OpCode::IReturn,
            0xad => OpCode::LReturn,
            0xae => OpCode::FReturn,
            0xaf => OpCode::DReturn,
            0xb0 => OpCode::AReturn,
            0xb1 => OpCode::Return,
            0xb2 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::GetStatic(constant_pool_index)
            }
            0xb3 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::PutStatic(constant_pool_index)
            }
            0xb4 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::GetField(constant_pool_index)
            }
            0xb5 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::PutField(constant_pool_index)
            }
            0xb6 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InvokeVirtual(constant_pool_index)
            }
            0xb7 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InvokeSpecial(constant_pool_index)
            }
            0xb8 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InvokeStatic(constant_pool_index)
            }
            0xb9 => {
                let constant_pool_index = load_u16(src)?;
                let _count = load_u8(src)?;
                let zero = load_u8(src)?;
                assert_eq!(zero, 0);
                curr_offset += 4;
                OpCode::InvokeInterface(constant_pool_index)
            }
            0xba => {
                let constant_pool_index = load_u16(src)?;
                let zeroes = load_u16(src)?;
                assert_eq!(zeroes, 0);
                curr_offset += 4;
                OpCode::InvokeDynamic(constant_pool_index)
            }
            0xbb => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::New(constant_pool_index)
            }
            0xbc => {
                let primitive_type = load_u8(src)?;
                curr_offset += 1;
                //println!("primitive_type:{}",primitive_type);
                OpCode::NewArray(primitive_type)
            }
            0xbd => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::ANewArray(constant_pool_index)
            }
            0xbe => OpCode::ArrayLength,
            0xbf => OpCode::Throw,
            0xc0 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::CheckCast(constant_pool_index)
            }
            0xc1 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InstanceOf(constant_pool_index)
            }
            0xc2 => OpCode::MonitorEnter,
            0xc3 => OpCode::MonitorExit,
            0xc4 => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Wide ops not supported!",
                ));
            }
            0xc5 => {
                let constant_pool_index = load_u16(src)?;
                let dimensions = load_u8(src)?;
                curr_offset += 3;
                OpCode::MultiANewArray(constant_pool_index, dimensions)
            }
            0xc6 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfNull(offset)
            }
            0xc7 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfNotNull(offset)
            }
            0xcb..=0xfd => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Invalid(reserved) opcode 0x{op:x}!"),
                ))
            } //OpCode::Reserved,
            _ => todo!("Unhandled opcode:0x{op:x}!"),
        };
        ops.push((decoded_op, op_offset));
        //println!("{decoded_op:?}");
    }
    //Check if last op is a return or throw(Useful sanity-check to catch some mistakes early.
    /*
    assert!(
    ops.iter().last().is_some_and(|lastop|{lastop.0.is_term()}) ||
     ops.iter().last().is_none()
    ,"ops:{ops:?}");*/
    //println!("ops:{ops:?}");
    Ok(ops)
}

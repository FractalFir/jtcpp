use super::{load_i16, load_i8, load_u16, load_u8};
#[derive(Debug, Clone, Copy)]
pub(crate) enum OpCode {
    Nop,
    ALoad(u8),
    FLoad(u8),
    ILoad(u8),
    DConst(f64),
    IConst(i32),
    AConstNull,
    AStore(u8),
    FStore(u8),
    IStore(u8),
    IAdd,
    FAdd,
    ISub,
    FSub,
    IMul,
    FMul,
    IDiv,
    IRem,
    IShr,
    IAnd,
    IInc(u8, i8),
    InvokeSpecial(u16),
    InvokeVirtual(u16),
    InvokeInterface(u16),
    InvokeStatic(u16),
    InvokeDynamic(u16),
    Return,
    IReturn,
    FReturn,
    LReturn,
    GetStatic(u16),
    PutStatic(u16),
    GetField(u16),
    PutField(u16),
    LoadConst(u16),
    IfICmpEq(i16),
    IfZero(i16),    //aka IfEq on wikipedia
    IfNotZero(i16), //aka IfEq on wikipedia
    IfNull(i16),
    IfNotNull(i16),
    IfIGreterEqual(i16),
    GoTo(i16),
    Dup,
    Pop,
    Pop2,
    New(u16),
    ANewArray(u16),
    BIPush,
    CheckCast(u16),
    ArrayLength,
    Throw,
    AALoad,
    BALoad,
    AAStore,
    AReturn,
}
pub(crate) fn load_ops<R: std::io::Read>(
    src: &mut R,
    code_length: u32,
) -> Result<Vec<(OpCode, u16)>, std::io::Error> {
    let mut curr_offset = 0;
    let mut ops = Vec::with_capacity(code_length as usize);
    while (curr_offset as u32) < code_length {
        let op = load_u8(src)?;
        let op_offset = curr_offset;
        //print!("{curr_offset}:\t");
        curr_offset += 1;
        let decoded_op = match op {
            0x0 => OpCode::Nop,
            0x1 => OpCode::AConstNull,
            0xf => OpCode::DConst(1.0),
            0x10 => OpCode::BIPush,
            0x2..=0x8 => OpCode::IConst(op as i32 - 0x3),
            0x12 => {
                let constant_pool_index = load_u8(src)?;
                curr_offset += 1;
                OpCode::LoadConst(constant_pool_index as u16)
            }
            0x13 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::LoadConst(constant_pool_index)
            }
            0x15 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::ILoad(index)
            }
            0x19 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::ALoad(index)
            }
            0x1a..=0x1d => OpCode::ILoad(op - 0x1a),
            0x22..=0x25 => OpCode::FLoad(op - 0x22),
            0x2a..=0x2d => OpCode::ALoad(op - 0x2a),
            0x32 => OpCode::AALoad,
            0x33 => OpCode::BALoad,
            0x3a => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::IStore(index)
            }
            0x3b..=0x3e => OpCode::IStore(op - 0x3b),
            0x43..=0x46 => OpCode::FStore(op - 0x43),
            0x4b..=0x4e => OpCode::AStore(op - 0x4b),
            0x36 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::IStore(index)
            }
            0x53 => OpCode::AAStore,
            0x57 => OpCode::Pop,
            0x58 => OpCode::Pop2,
            0x59 => OpCode::Dup,
            0x60 => OpCode::IAdd,
            0x62 => OpCode::FAdd,
            0x64 => OpCode::ISub,
            0x66 => OpCode::FSub,
            0x68 => OpCode::IMul,
            0x6a => OpCode::FMul,
            0x6c => OpCode::IDiv,
            0x70 => OpCode::IRem,
            0x7a => OpCode::IShr,
            0x7e => OpCode::IAnd,
            0x84 => {
                let var = load_u8(src)?;
                let incr = load_i8(src)?;
                curr_offset += 2;
                OpCode::IInc(var, incr)
            }
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
            0x9f => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfICmpEq(offset)
            }
            0xa2 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::IfIGreterEqual(offset)
            }
            0xa7 => {
                let offset = load_i16(src)?;
                curr_offset += 2;
                OpCode::GoTo(offset)
            }
            0xac => OpCode::IReturn,
            0xad => OpCode::LReturn,
            0xae => OpCode::FReturn,
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
                curr_offset += 2;
                OpCode::InvokeInterface(constant_pool_index)
            }
            0xba => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InvokeDynamic(constant_pool_index)
            }
            0xbb => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::New(constant_pool_index)
            }
            0xbe => OpCode::ArrayLength,
            0xbd => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::ANewArray(constant_pool_index)
            }
            0xbf => OpCode::Throw,
            0xc0 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::CheckCast(constant_pool_index)
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
            _ => todo!("Unhandled opcode:{op:x}!"),
        };
        ops.push((decoded_op, op_offset));
        //println!("{decoded_op:?}");
    }
    //println!("ops:{ops:?}");
    Ok(ops)
}

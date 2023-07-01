use crate::{FatOp,VariableType,IString,MethodCG};
macro_rules! basic_op_impl{
    ($vstack:ident,$cg:ident,$code:ident,$otype:literal,$op:literal) => {
        {
            let b = $vstack.pop().unwrap();
            let a = $vstack.pop().unwrap();
            let im_name = $cg.get_im_name();
            $code.push_str(&format!(concat!("\t",$otype," {} = {} ",$op," {};\n"),im_name,a,b));
            $vstack.push(im_name);
        }
    };
}
#[derive(Debug)]
pub(crate) struct BasicBlock<'a>{
    input:Vec<VariableType>,
    output:Vec<VariableType>,
    ops:&'a [FatOp],
    beg_idx:usize,
}
impl<'a> BasicBlock<'a>{
    pub(crate) fn new(
        ops:&'a [FatOp],
        beg_idx:usize)->Self{
        Self{input:Vec::new(),output:Vec::new(),ops,beg_idx}
    }
    fn vstack(&self)->Vec<IString>{
        Vec::new()
    }
    pub fn codegen(&self,cg:&mut MethodCG){
        let mut vstack = self.vstack();
        let mut code = String::new();
        for op in self.ops.iter(){
            match op{
                FatOp::FLoad(index)=>{
                    vstack.push(format!("loc{index}f").into());
                }
                FatOp::FMul=>basic_op_impl!(vstack,cg,code,"float","*"),
                FatOp::FAdd=>basic_op_impl!(vstack,cg,code,"float","+"),
                FatOp::FDiv=>basic_op_impl!(vstack,cg,code,"float","/"),
                FatOp::FSub=>basic_op_impl!(vstack,cg,code,"float","-"),
                FatOp::DMul=>basic_op_impl!(vstack,cg,code,"dobule","*"),
                FatOp::DAdd=>basic_op_impl!(vstack,cg,code,"dobule","+"),
                FatOp::DDiv=>basic_op_impl!(vstack,cg,code,"dobule","/"),
                FatOp::DSub=>basic_op_impl!(vstack,cg,code,"dobule","-"),
                FatOp::IMul=>basic_op_impl!(vstack,cg,code,"int","*"),
                FatOp::IAdd=>basic_op_impl!(vstack,cg,code,"int","+"),
                FatOp::IDiv=>basic_op_impl!(vstack,cg,code,"int","/"),
                FatOp::IRem=>basic_op_impl!(vstack,cg,code,"int","%"),
                FatOp::ISub=>basic_op_impl!(vstack,cg,code,"int","-"),
                FatOp::LMul=>basic_op_impl!(vstack,cg,code,"long","*"),
                FatOp::LAdd=>basic_op_impl!(vstack,cg,code,"long","+"),
                FatOp::LDiv=>basic_op_impl!(vstack,cg,code,"long","/"),
                FatOp::LSub=>basic_op_impl!(vstack,cg,code,"long","-"),
                FatOp::FConst(float)=>{
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tfloat {im_name} = {float};\n"));
                    vstack.push(im_name);
                },
                FatOp::IConst(int)=>{
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tint {im_name} = {int};\n"));
                    vstack.push(im_name);
                },
                FatOp::ILoad(var_idx)=>{
                    vstack.push(format!("loc{var_idx}i").into_boxed_str());
                },
                FatOp::IStore(var_idx)=>{
                    let vname = format!("loc{var_idx}i").into_boxed_str();
                    cg.ensure_exists(&vname,&VariableType::Int);
                    let set = vstack.pop().unwrap();
                    code.push_str(&format!("\t{vname} = {set};\n"));
                },
                FatOp::FReturn | FatOp::DReturn | FatOp::IReturn =>{
                    let ret = vstack.pop().unwrap();
                    code.push_str(&format!("\treturn {ret};\n"));
                }
                FatOp::IfNotZero(jump_pos)=>{
                    let val = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({val} != 0)goto bb_{jump_pos};\n"));
                }
                FatOp::IfIGreterEqual(jump_pos)=>{
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({a} >= {b})goto bb_{jump_pos};\n"));
                }
                FatOp::IfICmpGreater(jump_pos)=>{
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({a} > {b})goto bb_{jump_pos};\n"));
                }
                FatOp::GoTo(jump_pos)=>{
                    code.push_str(&format!("\tgoto bb_{jump_pos};\n"));
                },
                FatOp::IInc(variable,increment)=>{
                    let vname = format!("loc{variable}i").into_boxed_str();
                    cg.ensure_exists(&vname,&VariableType::Int);
                    code.push_str(&format!("\t{vname} = {vname} + {increment};\n"));
                }
                _=>todo!("Can't convert {op:?} to C."),
            }
        }
        println!("code:{code:?}");
        cg.put_bb(code.into(),self.beg_idx);
    }
}

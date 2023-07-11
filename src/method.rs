use crate::{BasicBlock,IString,VariableType,ImportedJavaClass,FatOp,MethodCG,method_desc_to_args};
pub(crate) struct Method {
    is_virtual:bool,
    class_name:IString,
    name: IString,
    ops: Box<[FatOp]>,
    args: Vec<VariableType>,
    ret_val: VariableType,
}
impl Method {
    pub(crate) fn ret_val(&self)->&VariableType{
        &self.ret_val
    }
    pub(crate) fn name(&self)->&str{
        &self.name
    }
    pub(crate) fn args(&self)->&[VariableType]{
        &self.args
    }
    pub(crate) fn from_raw_method(
        method: &crate::importer::Method,
        name: &str,
        jc: &ImportedJavaClass,
    ) -> Method {
        let name: IString = name.into();
        let (args, ret_val) = method_desc_to_args(method.descriptor(&jc));
        let is_virtual = method.is_virtual(jc);
        let ops = match method.bytecode() {
            Some(ops) => crate::fatops::expand_ops(ops, jc),
            None => [].into(),
        };
        Method {
            class_name:jc.lookup_class(jc.this_class()).unwrap().into(),
            is_virtual,
            name,
            args,
            ret_val,
            ops,
        }
    }
    fn into_bbs(&self) -> Vec<(usize, BasicBlock)> {
        let mut jump_targets = Vec::with_capacity(self.ops.len());
        for op in self.ops.iter() {
            match op.jump_target() {
                Some(targets) => {
                    targets.iter().for_each(|target| jump_targets.push(*target));
                }
                None => (),
            }
        }
        //println!("jump_targets:{jump_targets:?}");
        jump_targets.sort();
        jump_targets.dedup();
        let mut bbs = Vec::new();
        let mut bb_beg = 0;
        for (index, _op) in self.ops.iter().enumerate() {
            //println!("{index}:{op:?}");
            if jump_targets.contains(&index) {
                bbs.push((bb_beg, BasicBlock::new(&self.ops[bb_beg..index], bb_beg)));
                bb_beg = index;
            }
        }
        if bb_beg < self.ops.len() {
            bbs.push((bb_beg, BasicBlock::new(&self.ops[bb_beg..], bb_beg)));
        }
        bbs.into()
    }
    fn link_bbs(_bbs: &mut [(usize, BasicBlock)]) {
        //TODO:Link em
    }
    pub(crate) fn codegen(&self) -> IString {
        println!("Generating code for method {}",self.name);
        let mut bbs = self.into_bbs();
        Self::link_bbs(&mut bbs);
        let mut cg = MethodCG::new(&self.args, &self.name,&self.class_name, self.ret_val.clone(),self.is_virtual);
        for arg in &self.args{
            match arg.dependency(){
                Some(dep)=>cg.add_include(dep),
                None=>(),
            }
        }
        match self.ret_val.dependency(){
            Some(dep)=>cg.add_include(dep),
            None=>(),
        }
        for basic_block in bbs {
            basic_block.1.codegen(&mut cg);
        }
        cg.final_code()
    }
}
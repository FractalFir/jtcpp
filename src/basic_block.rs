use crate::{FatOp, IString, MethodCG, VariableType};
macro_rules! basic_op_impl {
    ($vstack:ident,$cg:ident,$code:ident,$otype:literal,$op:literal) => {{
        let b = $vstack.pop().unwrap();
        let a = $vstack.pop().unwrap();
        let im_name = $cg.get_im_name();
        $code.push_str(&format!(
            concat!("\t", $otype, " {} = {} ", $op, " {};\n"),
            im_name, a, b
        ));
        $vstack.push(im_name);
    }};
}
#[derive(Debug)]
pub(crate) struct BasicBlock<'a> {
    input: Vec<VariableType>,
    output: Vec<VariableType>,
    ops: &'a [FatOp],
    beg_idx: usize,
}
impl<'a> BasicBlock<'a> {
    pub(crate) fn new(ops: &'a [FatOp], beg_idx: usize) -> Self {
        Self {
            input: Vec::new(),
            output: Vec::new(),
            ops,
            beg_idx,
        }
    }
    fn vstack(&self) -> Vec<IString> {
        Vec::new()
    }
    pub fn codegen(&self, cg: &mut MethodCG) {
        let mut vstack = self.vstack();
        let mut code = String::new();
        for op in self.ops.iter() {
            match op {
                FatOp::FLoad(index) => vstack.push(format!("loc{index}f").into()),
                FatOp::ALoad(index) => vstack.push(format!("loc{index}a").into()),
                FatOp::FMul => basic_op_impl!(vstack, cg, code, "float", "*"),
                FatOp::FRem => {
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tfloat {im_name} = fmodf({a},{b});\n"));
                    vstack.push(im_name);
                },
                FatOp::ANewArray(class)=>{
                    let im_name = cg.get_im_name();
                    let length = vstack.pop().unwrap();
                    code.push_str(&format!("\tstd::shared_ptr<RuntimeArray<std::shared_ptr<{class}>>> {im_name}aa = std::shared_ptr<RuntimeArray<std::shared_ptr<{class}>>>(new RuntimeArray<std::shared_ptr<{class}>>({length}));\n"));
                    vstack.push(format!("{im_name}aa").into());
                },
                FatOp::FAdd => basic_op_impl!(vstack, cg, code, "float", "+"),
                FatOp::FDiv => basic_op_impl!(vstack, cg, code, "float", "/"),
                FatOp::FSub => basic_op_impl!(vstack, cg, code, "float", "-"),
                FatOp::DMul => basic_op_impl!(vstack, cg, code, "dobule", "*"),
                FatOp::DAdd => basic_op_impl!(vstack, cg, code, "dobule", "+"),
                FatOp::DDiv => basic_op_impl!(vstack, cg, code, "dobule", "/"),
                FatOp::DSub => basic_op_impl!(vstack, cg, code, "dobule", "-"),
                FatOp::IMul => basic_op_impl!(vstack, cg, code, "int", "*"),
                FatOp::IAdd => basic_op_impl!(vstack, cg, code, "int", "+"),
                FatOp::IDiv => basic_op_impl!(vstack, cg, code, "int", "/"),
                FatOp::IRem => basic_op_impl!(vstack, cg, code, "int", "%"),
                FatOp::ISub => basic_op_impl!(vstack, cg, code, "int", "-"),
                FatOp::LMul => basic_op_impl!(vstack, cg, code, "long", "*"),
                FatOp::LAdd => basic_op_impl!(vstack, cg, code, "long", "+"),
                FatOp::LDiv => basic_op_impl!(vstack, cg, code, "long", "/"),
                FatOp::LSub => basic_op_impl!(vstack, cg, code, "long", "-"),
                FatOp::FConst(float) => {
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tfloat {im_name} = {float:?}f;\n"));
                    vstack.push(im_name);
                }
                FatOp::IConst(int) => {
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tint {im_name} = {int};\n"));
                    vstack.push(im_name);
                }
                FatOp::SConst(short) => {
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tshort {im_name} = {short};\n"));
                    vstack.push(im_name);
                }
                FatOp::StringConst(string) => {
                    let im_name = cg.get_im_name();
                    code.push_str("\t const short[] ");
                    code.push_str(&im_name);
                    code.push_str("_data = [");

                    for c in string.encode_utf16() {
                        code.push_str(&format!("{c:x},"));
                    }
                    code.push_str(&format!("0x0000];\n java_cs_lang_cs_String {im_name} = runtime_alloc_string(im_data,sizeof({im_name}_data)/sizeof({im_name}_data[]0);\n"));
                    vstack.push(im_name);
                }
                FatOp::ILoad(var_idx) => {
                    vstack.push(format!("loc{var_idx}i").into_boxed_str());
                }
                FatOp::IStore(var_idx) => {
                    let vname = format!("loc{var_idx}").into_boxed_str();
                    cg.ensure_exists(&vname, &VariableType::Int);
                    let set = vstack.pop().unwrap();
                    code.push_str(&format!("\t{vname}i = {set};\n"));
                }
                FatOp::FStore(var_idx) => {
                    let vname = format!("loc{var_idx}").into_boxed_str();
                    cg.ensure_exists(&vname, &VariableType::Float);
                    let set = vstack.pop().unwrap();
                    code.push_str(&format!("\t{vname}f = {set};\n"));
                }
                FatOp::Dup => {
                    let val = vstack.pop().unwrap();
                    vstack.push(val.clone());
                    vstack.push(val);
                }
                FatOp::Pop => {vstack.pop().unwrap();},
                FatOp::New(class_name) => {
                    let im_name = cg.get_im_name();
                    cg.ensure_exists(&im_name,&VariableType::ObjectRef{name:class_name.clone()});
                    code.push_str(&format!(
                        "\t{im_name}a = std::shared_ptr<{class_name}>(new {class_name}());\n"
                    ));
                    vstack.push(format!("{im_name}a").into());
                }
                FatOp::ArrayLength=>{
                    let array = vstack.pop().unwrap();
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tint {im_name} = {array}.GetLength();\n"));
                    vstack.push(im_name);
                }
                FatOp::AStore(var_idx) => {
                    let vname = format!("loc{var_idx}").into_boxed_str();
                    cg.ensure_exists(&vname, &VariableType::Int);
                    let set = vstack.pop().unwrap();
                    code.push_str(&format!("\t{vname}a = {set};\n"));
                }
                FatOp::FReturn | FatOp::DReturn | FatOp::IReturn | FatOp::AReturn => {
                    let ret = vstack.pop().unwrap();
                    code.push_str(&format!("\treturn {ret};\n"));
                }
                FatOp::Return => code.push_str("\treturn;\n"),
                FatOp::InvokeVirtual(_class_name,vmethod_name, args, ret) => {
                    let mut args:Vec<_> = args.iter().map(|_|{vstack.pop().unwrap()}).collect();
                    args.push(vstack.pop().unwrap());
                    args.reverse();
                    let mut args = args.iter();
                    let objref = args.next().unwrap();
                    
                    if *ret == VariableType::Void{
                        code.push_str(&format!("\t{objref}->{vmethod_name}("));
                    }
                    else{
                        let im_name = cg.get_im_name();
                        code.push_str(&format!("\t{ret} {im_name} = {objref}->{vmethod_name}(",ret = ret.c_type()));
                        vstack.push(im_name);
                    }
                    match args.next(){
                        Some(arg)=>code.push_str(arg),
                        None=>(),
                    }
                    for arg in args{
                        code.push(',');
                        code.push_str(arg);
                    }
                    code.push_str(");\n");
                }
                FatOp::AALoad=>{
                    let index = vstack.pop().unwrap();
                    let array_ref = vstack.pop().unwrap();
                    let im_name = cg.get_im_name();
                    cg.ensure_exists_auto(&im_name);
                    code.push_str(&format!("\t{im_name} = {array_ref}->Get({index});\n"));
                    vstack.push(im_name);
                },
                FatOp::FGetStatic(class_name,static_name)=>{
                    let im_name = cg.get_im_name();
                    cg.add_include(class_name);
                    code.push_str(&format!("\tfloat {im_name} = {class_name}::{static_name};\n"));
                    vstack.push(im_name);
                }
                FatOp::AGetStatic{class_name,static_name,type_name}=>{
                    let im_name = cg.get_im_name();
                    cg.add_include(class_name);
                    cg.ensure_exists(&im_name,&VariableType::ObjectRef{name:type_name.clone()});
                    code.push_str(&format!("\t{im_name} = {class_name}::{static_name};\n"));
                    vstack.push(im_name);
                }
                FatOp::FPutStatic(class_name,static_name)=>{
                    let set = vstack.pop().unwrap();
                    cg.add_include(class_name);
                    code.push_str(&format!("\t{class_name}_{static_name} = {set};\n"));
                }
                FatOp::InvokeStatic(method_class_name,method_name,args,ret) => {
                    cg.add_include(method_class_name);
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\t{ret_ctype} {im_name} = {method_class_name}::{method_name}(",ret_ctype = ret.c_type()));
                    let mut args = args.into_iter().enumerate();
                    match args.next(){
                        Some((_,_))=>{
                            let val = vstack.pop().unwrap();
                            code.push_str(&val);
                        } 
                        None=>(),
                    }
                    for _ in args{
                        let val = vstack.pop().unwrap();
                        code.push(',');
                        code.push_str(&val);
                    }
                    code.push_str(");\n");
                    vstack.push(im_name);
                }
                FatOp::IfNotZero(jump_pos) => {
                    let val = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({val} != 0)goto bb_{jump_pos};\n"));
                }
                FatOp::IfIGreterEqual(jump_pos) => {
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({a} >= {b})goto bb_{jump_pos};\n"));
                }
                FatOp::IfICmpGreater(jump_pos) => {
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({a} > {b})goto bb_{jump_pos};\n"));
                }
                FatOp::IfICmpNe(jump_pos) => {
                    let b = vstack.pop().unwrap();
                    let a = vstack.pop().unwrap();
                    code.push_str(&format!("\tif({a} != {b})goto bb_{jump_pos};\n"));
                }
                FatOp::GoTo(jump_pos) => {
                    code.push_str(&format!("\tgoto bb_{jump_pos};\n"));
                }
                FatOp::IInc(variable, increment) => {
                    let vname = format!("loc{variable}").into_boxed_str();
                    cg.ensure_exists(&vname, &VariableType::Int);
                    code.push_str(&format!("\t{vname}i = {vname}i + {increment};\n"));
                }
                FatOp::APutField {
                    class_name: _,
                    field_name,
                    type_name,
                } => {
                    let objref = vstack.pop().unwrap();
                    let field_owner = vstack.pop().unwrap();
                    cg.add_include(type_name);
                    code.push_str(&format!("\t{field_owner}->{field_name} = {objref};\n"));
                }
                FatOp::AAPutField{class_name,field_name,atype}=>{
                    let arrref = vstack.pop().unwrap();
                    let field_owner = vstack.pop().unwrap();
                    cg.add_include(class_name);
                    if let Some(dep) = atype.dependency(){
                        cg.add_include(dep);
                    }
                    code.push_str(&format!("\t{field_owner}->{field_name} = {arrref};\n"));
                },
                FatOp::AAGetField{class_name,field_name,atype}=>{
                    let field_owner = vstack.pop().unwrap();
                    cg.add_include(class_name);
                    if let Some(dep) = atype.dependency(){
                        cg.add_include(dep);
                    }
                    let im_name = cg.get_im_name();
                    cg.ensure_exists(&im_name,&VariableType::ArrayRef(Box::new(atype.clone())));
                    code.push_str(&format!("\t{im_name}aa = {field_owner}->{field_name};\n"));
                    vstack.push(format!("{im_name}aa").into());
                },
                FatOp::F2D=>{
                    let float = vstack.pop().unwrap();
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tdouble {im_name} = (double){float};\n"));
                    vstack.push(im_name);
                }
                FatOp::D2F=>{
                    let double = vstack.pop().unwrap();
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tfloat {im_name} = (float){double};\n"));
                    vstack.push(im_name);
                }
                FatOp::FGetField(_class_name,field_name)=>{
                    let field_owner = vstack.pop().unwrap();
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tfloat {im_name} = {field_owner}->{field_name};\n"));
                    vstack.push(im_name);
                },
                FatOp::AGetField{class_name: _,field_name,type_name}=>{
                    let field_owner = vstack.pop().unwrap();
                    let im_name = cg.get_im_name();
                    code.push_str(&format!("\tstd::shared_ptr<{type_name}> {im_name} = {field_owner}->{field_name};\n"));
                    vstack.push(im_name);
                },
                FatOp::AAStore=>{
                    let value = vstack.pop().unwrap();
                    let index = vstack.pop().unwrap();
                    let array_ref = vstack.pop().unwrap();
                    code.push_str(&format!("\t{array_ref}->Set({index},{value});\n"));
                },
                FatOp::FPutField(_class_name, field_name) => {
                    let float_value = vstack.pop().unwrap();
                    let field_owner = vstack.pop().unwrap();
                    code.push_str(&format!("\t{field_owner}->{field_name} = {float_value};\n"));
                }
                FatOp::InvokeSpecial(class_path, method_name, mut argc) => {
                    if method_name.contains("_init_") {
                        argc += 1;
                    }
                    let mut args = Vec::with_capacity(argc as usize);
                    for _ in 0..argc {
                        args.push(vstack.pop().unwrap());
                    }
                    args.reverse();
                    //let im_name = cg.get_im_name();
                    cg.add_include(class_path);
                    code.push('\t');
                    code.push_str(method_name);
                    code.push('(');
                    let mut args = args.iter();
                    match args.next() {
                        Some(arg) => code.push_str(arg),
                        None => (),
                    }
                    for arg in args {
                        code.push(',');
                        code.push_str(arg);
                    }
                    code.push_str(");\n");
                }
                _ => todo!("Can't convert {op:?} to C."),
            }
        }
        //println!("code:{code:?}");
        cg.put_bb(code.into(), self.beg_idx);
    }
}

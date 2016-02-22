use super::LLVMValue;
use super::LLVMContext;
use super::LLVMModule;
use torrentlearn_model::parse::SingleOperators;

extern {

        pub fn get_global_context() -> *mut u8;
        pub fn create_jit() -> *mut u8;
        pub fn create_IRBuilder(context: *mut u8) -> *mut u8;
        pub fn initialize_module(context: *mut u8,jit: *mut u8)-> *mut u8;
        pub fn initialize_pass_manager(module: *mut u8)-> *mut u8;
        pub fn generate_constant(context: *mut u8, val: u64)-> *mut u8;
        pub fn extern_generate_function_proto(context: *mut u8, module: *mut u8)-> FunctionProto;
        pub fn extern_finalize_function(context: *mut u8,ir_builder: *mut u8,function: *mut u8)->*mut u8;
        pub fn extern_load_array_cell(context: *mut u8,ir_builder: *mut u8,val: u8, array: *mut u8)->*mut u8;
        //pub fn extern_drop_value(value: *mut u8);
}

#[repr(C)]
pub struct FunctionProto {
    proto: *mut u8,
    args: [*mut u8;10]
}

pub fn initialize_llvm() -> LLVMContext
{
    let c; let j; let f;
    unsafe{
        // Get the global LLVM context, jit and ir builder
      c = get_global_context();
      j = create_jit();
      f = create_IRBuilder(c);
  }

  LLVMContext{context: c, kaleidoscope_jit:j, ir_builder:f}

}

pub fn initialize_llvm_module(context: &mut LLVMContext) -> LLVMModule
{
    let m; let c;
    unsafe{
       // Create a new module for our function.
       m = initialize_module(context.context,context.kaleidoscope_jit);

       c = initialize_pass_manager(m);

   }
   LLVMModule{module: m, function_pass_analyzer: c}

}

/*pub fn drop_value(val: &mut LLVMValue)
{
    let &mut LLVMValue(internal)=val;
    unsafe{
        extern_drop_value(internal);
    }
}*/
pub fn generate_constant_val(context: &mut LLVMContext, val: u64) -> LLVMValue
{
    unsafe{
        LLVMValue(generate_constant(context.context,val))
    }
}
pub fn generate_function_proto(context: &mut LLVMContext,module: &mut LLVMModule) -> (LLVMValue,Vec<LLVMValue>)
{
    unsafe
    {
        let proto: FunctionProto = extern_generate_function_proto(context.context, module.module);
        let mut argument_vec = Vec::new();
        for value in proto.args.iter()
        {
            argument_vec.push(LLVMValue(*value));
        }
        return (LLVMValue(proto.proto),argument_vec)
    }
}
pub fn finalize_function(val: LLVMValue, context: &mut LLVMContext, _: &mut LLVMModule) -> LLVMValue
{
    let LLVMValue(val)= val;
    unsafe{
        LLVMValue(extern_finalize_function(context.context,context.ir_builder,val))
    }
}
pub fn load_array_cell(context: &mut LLVMContext, _: &mut LLVMModule, array: &mut LLVMValue, val: u8) -> LLVMValue
{
    let &mut LLVMValue(array)= array;
    unsafe{
        LLVMValue(extern_load_array_cell(context.context,context.ir_builder,val,array))
    }
}

pub fn generate_single_statement(operator: SingleOperators, dest: LLVMValue,source:LLVMValue) -> LLVMValue
{
    unimplemented!()
}



#[cfg(test)]
mod tests {

    static Operators: [SingleOperators; 1] = [Add];
    use parse::SingleOperators;

    #[test]
    fn null_and_sanity_checks_extern() {
        unsafe{
            let context = super::get_global_context();
            let jit = super::create_jit();
            let ir_builder = super::create_IRBuilder(context);
            assert!(!context.is_null());
            assert!(!jit.is_null());
            assert!(!ir_builder.is_null());

            let module = super::initialize_module(context,jit);
            assert!(!module.is_null());

            let fpm = super::initialize_pass_manager(module);
            assert!(!module.is_null());
        }


    }
}

use torrentlearn_model::SpecialOperator;
use torrentlearn_model::Operator;
use torrentlearn_model::parse::AllOperators;
use torrentlearn_model::parse::ParseTree;
use codegen::{LLVMContext,LLVMModule,LLVMJit,LLVMIrBuilder};
use codegen::llvminterface;
use codegen;
use torrentlearn_model::operator::DropHelper;
use std::sync::{Arc,Mutex};

extern crate core;

pub struct JitCompiler {
    context: LLVMContext,
    jit: Arc<Mutex<LLVMJit>>,
    ir_builder: LLVMIrBuilder,
    count: u64
}


impl JitCompiler {
    pub fn new(rotate_period: u16) -> JitCompiler {
        let (context,jit,ir_builder) =llvminterface::initialize_llvm();
        assert!(rotate_period!=0);
        JitCompiler{context: context, jit: Arc::new(Mutex::new(jit)), ir_builder: ir_builder, count: 0}
    }
    //FIXME: Rotation logic isn't implemented and every module has one functions
    pub fn compile_operator(&mut self,mut parse_tree: ParseTree, base_cost_calculator: fn(&AllOperators)-> u64, combination_cost_calculator: fn(u64,u64)-> u64) -> Operator {
        let pointer;
        let mut jit = self.jit.lock().unwrap();
        let (mut module,mut fpm) = LLVMModule::with_context(&mut self.context, &mut jit );
        pointer = codegen::compile(&mut jit, &mut self.context, &mut self.ir_builder, &mut module, &mut fpm, &mut parse_tree, &("operator_".to_string() + &self.count.to_string())).unwrap();

        self.count = self.count +1;
        let compiledmodule = Box::new(codegen::CompiledModule{module: Arc::new(module), jit:self.jit.clone()}) as Box<DropHelper>;
        Operator{ special: SpecialOperator::None, successors: parse_tree.get_sucessors(),cost: parse_tree.calculate_cost(base_cost_calculator,combination_cost_calculator), op: pointer, parts: Some(Arc::new(parse_tree)), drop_helper: Some(compiledmodule) }
    }
}

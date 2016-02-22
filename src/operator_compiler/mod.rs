use torrentlearn_model::SpecialOperator;
use torrentlearn_model::Operator;
use torrentlearn_model::parse::AllOperators;
use torrentlearn_model::parse::ParseTree;
use codegen::{LLVMContext,LLVMModule};
use codegen::llvminterface;
use codegen;
use std::sync::{Arc,Mutex};

extern crate core;

pub struct JitCompiler {
    context: LLVMContext,
    current_module: Arc<Mutex<LLVMModule>>,
    rotate_count: u16,
    rotate_period: u16
}

impl JitCompiler {
    pub fn new(rotate_period: u16) -> JitCompiler {
        let mut context =llvminterface::initialize_llvm();
        let module = Arc::new(Mutex::new(LLVMModule::with_context(&mut context)));
        assert!(rotate_period!=0);
        JitCompiler{context: context, current_module: module, rotate_period: rotate_period, rotate_count: 0}
    }
    //FIXME: Rotation logic isnt clear with the multiple modules
    pub fn compile_operator(&mut self,mut parse_tree: ParseTree, base_cost_calculator: fn(&AllOperators)-> u64, combination_cost_calculator: fn(u64,u64)-> u64) -> Operator {
        let (pointer, current_module) = codegen::compile(&mut self.context, &mut self.current_module, &mut parse_tree);
        if self.rotate_count % self.rotate_period ==0 {
            self.rotate_module()
        }
        self.rotate_count = (self.rotate_count % self.rotate_period) +1;
        Operator{ special: SpecialOperator::None, successors: parse_tree.get_sucessors(),cost: parse_tree.calculate_cost(base_cost_calculator,combination_cost_calculator), op: pointer, parts: Some(Arc::new(parse_tree)), drop_helper: Some(current_module) }
    }

    pub fn rotate_module(&mut self) {
        self.current_module= Arc::new(Mutex::new(LLVMModule::with_context(&mut self.context)));
    }
}

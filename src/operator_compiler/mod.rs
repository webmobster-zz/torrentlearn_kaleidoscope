use torrentlearn_model::SpecialOperator;
use torrentlearn_model::Operator;
use torrentlearn_model::parse::AllOperators;
use torrentlearn_model::parse::ParseTree;
use codegen::{LLVMContext,LLVMModule};
use codegen::llvminterface;
use codegen;
use torrentlearn_model::operator::DropHelper;
use std::sync::{Arc,Mutex};

extern crate core;

pub struct JitCompiler {
    context: LLVMContext,
    current_module: Arc<Mutex<Box<LLVMModule>>>,
    rotate_count: u16,
    rotate_period: u16,
    count: u64
}

impl JitCompiler {
    pub fn new(rotate_period: u16) -> JitCompiler {
        let mut context =llvminterface::initialize_llvm();
        let module = Arc::new(Mutex::new(Box::new(LLVMModule::with_context(&mut context))));
        assert!(rotate_period!=0);
        JitCompiler{context: context, current_module: module, rotate_period: rotate_period, rotate_count: 0, count: 0}
    }
    //FIXME: Rotation logic isn't implemented and every module has one functions
    pub fn compile_operator(&mut self,mut parse_tree: ParseTree, base_cost_calculator: fn(&AllOperators)-> u64, combination_cost_calculator: fn(u64,u64)-> u64) -> Operator {
        let pointer;
        {
            let module = &mut self.current_module.lock().unwrap();
            //FIXME this sting is fucked
            pointer = codegen::compile(&mut self.context, module, &mut parse_tree, &("operator_".to_string() + &self.count.to_string())).unwrap();
        }
        //if self.rotate_count % self.rotate_period == 0 {
        if true {
            self.rotate_module()
        }
        self.rotate_count = (self.rotate_count % self.rotate_period) +1;
        self.count = self.count +1;
        Operator{ special: SpecialOperator::None, successors: parse_tree.get_sucessors(),cost: parse_tree.calculate_cost(base_cost_calculator,combination_cost_calculator), op: pointer, parts: Some(Arc::new(parse_tree)), drop_helper: Some(self.current_module.clone() as Arc<Mutex<DropHelper + Send>>) }
    }

    pub fn rotate_module(&mut self) {
        self.current_module= Arc::new(Mutex::new(Box::new(LLVMModule::with_context(&mut self.context))));
    }
}

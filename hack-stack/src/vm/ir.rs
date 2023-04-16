use std::collections::{HashMap, HashSet, VecDeque};

use crate::common::SourceFile;

use super::ast;

pub enum Instruction<'a> {
    SimpleInstruction(ast::Instruction<'a>),
}

pub struct Function<'a> {
    pub name: &'a str,
    pub instructions: Vec<Instruction<'a>>,
    pub source_file: &'a SourceFile,
}

pub struct Module<'a> {
    pub name: &'a str,
    pub instructions: Vec<Instruction<'a>>,
    pub source_file: &'a SourceFile,
}

pub struct Program<'a> {
    pub functions: HashMap<String, Function<'a>>,
    pub modules: HashMap<String, Module<'a>>,
    pub reachable_functions: HashSet<String>,
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            modules: HashMap::new(),
            reachable_functions: HashSet::new(),
        }
    }

    pub fn add_module(
        &mut self,
        instructions: Vec<ast::Instruction<'a>>,
        source_file: &'a SourceFile,
    ) {
        let mut func: Option<Function> = None;
        let mut module_instructions = vec![];
        for inst in instructions {
            match inst {
                ast::Instruction::Function(fn_instruction) => {
                    if let Some(last_func) = func {
                        self.functions.insert(last_func.name.to_string(), last_func);
                    }
                    func = Some(Function {
                        name: fn_instruction.name,
                        instructions: vec![Instruction::SimpleInstruction(
                            ast::Instruction::Function(fn_instruction),
                        )],
                        source_file,
                    });
                }
                inst => {
                    let ir_inst = Instruction::SimpleInstruction(inst);
                    if let Some(func) = func.as_mut() {
                        func.instructions.push(ir_inst);
                    } else {
                        module_instructions.push(ir_inst);
                    }
                }
            }
        }
        if let Some(func) = func {
            self.functions.insert(func.name.to_string(), func);
        }
        self.modules.insert(
            source_file.name.clone(),
            Module {
                name: &source_file.name,
                instructions: module_instructions,
                source_file,
            },
        );
    }

    pub fn mark_reachable_functions(&mut self) {
        let mut func_queue = VecDeque::new();
        func_queue.push_back("Sys.init");

        // Top-level code in modules can also be considered entrypoints. Programs compiled from
        // Jack shouldn't have any.
        for module in self.modules.values() {
            for inst in &module.instructions {
                if let Instruction::SimpleInstruction(ast::Instruction::Call(call)) = inst {
                    func_queue.push_back(call.function);
                }
            }
        }

        while !func_queue.is_empty() {
            let func_name = func_queue.pop_front().unwrap();
            if self.reachable_functions.contains(func_name) {
                continue;
            }
            self.reachable_functions.insert(func_name.to_string());

            let Some(func) = self.functions.get(func_name) else {
                continue;
            };

            for inst in &func.instructions {
                if let Instruction::SimpleInstruction(ast::Instruction::Call(call)) = inst {
                    func_queue.push_back(call.function);
                }
            }
        }
    }

    pub fn print_call_tree(&self) {
        let Some(func) = self.functions.get("Sys.init") else {
            return;
        };

        let mut func_queue = VecDeque::new();
        let mut func_stack = Vec::new();
        func_queue.push_back((func, 0));

        while !func_queue.is_empty() {
            let (func, depth) = func_queue.pop_front().unwrap();
            println!("{}{}", "  ".repeat(depth), func.name);

            while func_stack.len() > depth {
                func_stack.pop();
            }
            func_stack.push(func.name);

            let mut seen = HashSet::new();
            for inst in &func.instructions {
                if let Instruction::SimpleInstruction(ast::Instruction::Call(call)) = inst {
                    if !func_stack.contains(&call.function) && !seen.contains(&call.function) {
                        let func = self.functions.get(call.function).unwrap();
                        func_queue.push_front((func, depth + 1));
                        seen.insert(call.function);
                    }
                }
            }
        }
    }
}

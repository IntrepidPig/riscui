mod utils;

use risclang::Instruction;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use std::{convert::{TryFrom}};
#[wasm_bindgen(module = "src/lib/shims")]
extern {
    fn wasm_print(text: &str);
}

pub fn print(text: String) {
    wasm_print(&text)
}

#[wasm_bindgen]
pub fn test() -> u32 {
    return 200;
}

#[derive(Serialize)]
pub struct CodeItem {
    code: u32,
    text: String,
}

#[wasm_bindgen]
pub fn compile(source: &str) -> JsValue {
    let (insts, texts, labels) = risclang::parse::parse(source);
    let code = risclang::compile::compile(insts, &labels);
    assert_eq!(code.len(), texts.len());
    let items = (0..code.len()).map(|i| CodeItem { code: code[i].0, text: texts[i].clone() }).collect::<Vec<_>>();
    serde_wasm_bindgen::to_value(&items).unwrap()
}

#[wasm_bindgen]
pub struct Machine {
    inner: riscvm::Machine,
}

#[wasm_bindgen]
pub struct ExecResult {
    pub a0: i32,
    pub a1: i32,
}

#[wasm_bindgen]
impl Machine {
    pub fn new(memory: usize) -> Self {
        utils::set_panic_hook();
        let inner = riscvm::Machine::new(memory);
        Self {
            inner,
        }
    }
    
    pub fn get_instruction_index(&self) -> usize {
        print(format!("pc {}", self.inner.pc));
        usize::try_from(self.inner.pc).unwrap() / 4
    }
    
    pub fn get_registers(&self) -> Vec<i32> {
        self.inner.regs.to_vec()
    }
    
    pub fn get_memory_view(&self, start: usize, len: usize) -> Vec<u8> {
        self.inner.mem[start..(start + len)].to_vec()
    }
    
    pub fn exec(&mut self, inst: u32) -> Option<ExecResult> {
        print(format!("executing {}", inst));
        self.inner.exec(Instruction(inst)).map(|x| ExecResult { a0: x.0, a1: x.1 })
    }
}
mod utils;

use risclang::Instruction;
use wasm_bindgen::prelude::*;
use std::convert::TryFrom;
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

#[wasm_bindgen]
pub fn compile(source: &str) -> Vec<u32> {
    let (insts, labels) = risclang::parse::parse(source);
    risclang::compile::compile(insts, &labels).into_iter().map(|x| x.0).collect()
}

#[wasm_bindgen]
pub struct Machine {
    inner: riscvm::Machine,
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
    
    pub fn exec(&mut self, inst: u32) -> bool {
        print(format!("executing {}", inst));
        self.inner.exec(Instruction(inst));
        true
    }
}
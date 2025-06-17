use anyhow::Result;

use crate::{parser::ast::Immediate, vm::Error};

#[derive(Debug)]
pub struct Stack {
    pub(crate) storage: Vec<Immediate>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Immediate) -> Result<()> {
        if self.storage.len() as isize >= isize::MAX {
            return Err(Error::StackOverflow.into());
        }
        self.storage.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Immediate> {
        if let Some(imm) = self.storage.pop() {
            Ok(imm)
        } else {
            Err(Error::StackUnderflow.into())
        }
    }
}

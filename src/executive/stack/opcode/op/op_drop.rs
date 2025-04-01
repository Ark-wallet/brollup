use crate::executive::stack::stack::{Stack, StackError};

/// The `OP_DROP` opcode.
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct OP_DROP;

impl OP_DROP {
    pub fn execute(stack: &mut Stack) -> Result<(), StackError> {
        // Pop the last item from stack.
        stack.pop().ok_or(StackError::EmptyStack)?;

        Ok(())
    }
}

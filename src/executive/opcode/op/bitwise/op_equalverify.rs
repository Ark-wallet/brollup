use crate::executive::{
    opcode::ops::OP_EQUALVERIFY_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};

/// Pops two items from the main stack and checks if they are equal. Fails if they are not.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_EQUALVERIFY;

impl OP_EQUALVERIFY {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop two items from the main stack.
        let item_1 = stack_holder.pop()?;
        let item_2 = stack_holder.pop()?;

        // Check if the two items are equal.
        if item_1.bytes() != item_2.bytes() {
            return Err(StackError::MandatoryEqualVerifyError);
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_EQUALVERIFY_OPS)?;

        Ok(())
    }
}

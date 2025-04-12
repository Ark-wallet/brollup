use crate::executive::{
    opcode::ops::OP_MULMOD_OPS,
    stack::{
        stack_error::StackError,
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{StackItemUintExt, StackUint},
    },
};

/// Multiplies two items on the main stack and returns the result modulo MAX::U256.
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct OP_MULMOD;

impl OP_MULMOD {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop two items from the main stack.
        let item_1 = stack_holder.pop()?;
        let item_2 = stack_holder.pop()?;

        // Irem 1 uint value;
        let item_1_uint = item_1
            .to_uint()
            .ok_or(StackError::StackUintMaxOverflowError)?;

        // Item 2 uint value;
        let item_2_uint = item_2
            .to_uint()
            .ok_or(StackError::StackUintMaxOverflowError)?;

        // Multiply the two values modulo MAX::U256.
        let result = StackItem::from_uint(StackUint::mulmod(&item_1_uint, &item_2_uint));

        // Increment the ops counter.
        stack_holder.increment_ops(OP_MULMOD_OPS)?;

        // Push the result to the main stack.
        stack_holder.push(result)?;

        Ok(())
    }
}

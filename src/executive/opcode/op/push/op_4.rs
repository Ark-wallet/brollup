use crate::executive::{
    opcode::{
        codec::{OpcodeEncoder, OpcodeEncoderError},
        ops::OP_4_OPS,
    },
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};

/// Pushes number 4 (0x04) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_4;

impl OP_4 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push 4 (0x04) to the main stack.
        let item_to_push = StackItem::new(vec![0x04]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_4_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }
}

/// Implement the `OpcodeEncoder` trait for `OP_4`.
impl OpcodeEncoder for OP_4 {
    fn encode(&self) -> Result<Vec<u8>, OpcodeEncoderError> {
        Ok(vec![0x54])
    }
}

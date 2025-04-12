use crate::executive::{
    opcode::{
        codec::{OpcodeEncoder, OpcodeEncoderError},
        ops::OP_IF_OPS,
    },
    stack::{
        flow::{flow_encounter::FlowEncounter, flow_status::FlowStatus},
        stack_error::StackError,
        stack_holder::StackHolder,
    },
};

/// The `OP_IF` opcode.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_IF;

impl OP_IF {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // Increment the ops counter.
        stack_holder.increment_ops(OP_IF_OPS)?;

        // If this is not the active execution, return with uncovered.
        if !stack_holder.active_execution() {
            stack_holder.push_flow_encounter(FlowEncounter::IfNotif(FlowStatus::Uncovered));
            return Ok(());
        }

        // Pop the latest item from the stack.
        let item = stack_holder.pop()?;

        // If the item is not true, set the active execution flag to false.
        if !item.is_true() {
            stack_holder.push_flow_encounter(FlowEncounter::IfNotif(FlowStatus::Inactive));
        } else {
            stack_holder.push_flow_encounter(FlowEncounter::IfNotif(FlowStatus::Active));
        }

        Ok(())
    }
}

/// Implement the `OpcodeEncoder` trait for `OP_IF`.
impl OpcodeEncoder for OP_IF {
    fn encode(&self) -> Result<Vec<u8>, OpcodeEncoderError> {
        Ok(vec![0x63])
    }
}

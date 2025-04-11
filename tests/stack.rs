#[cfg(test)]
mod stack_tests {

    use brollup::executive::{
        opcode::op::{
            altstack::{op_fromaltstack::OP_FROMALTSTACK, op_toaltstack::OP_TOALTSTACK},
            arithmetic::op_add::OP_ADD,
            bitwise::op_equalverify::OP_EQUALVERIFY,
            flow::{op_else::OP_ELSE, op_endif::OP_ENDIF, op_if::OP_IF, op_verify::OP_VERIFY},
            push::{op_2::OP_2, op_3::OP_3, op_4::OP_4, op_false::OP_FALSE, op_true::OP_TRUE},
            splice::op_cat::OP_CAT,
        },
        stack::{
            stack::{Stack, StackHolder},
            stack_error::StackError,
            stack_item::{
                item::StackItem,
                uint_ext::{StackItemUintExt, StackUint},
            },
        },
    };

    #[test]
    fn stack_test() -> Result<(), StackError> {
        let mut stack_holder = StackHolder::new([0; 32], [0; 32], 50, 0);

        // Initialize main stack.

        // Push 0xdeadbeef
        let _ = stack_holder.push(StackItem::new(vec![0xde, 0xad, 0xbe, 0xef]));

        // Push 0xdead
        let _ = stack_holder.push(StackItem::new(vec![0xde, 0xad]));

        // Push 0xbeef
        let _ = stack_holder.push(StackItem::new(vec![0xbe, 0xef]));

        // Print the stack.
        println!("Stack: {}", stack_holder.stack());

        // OP_TOALTSTACK
        OP_TOALTSTACK::execute(&mut stack_holder)?;

        // Print the stack.
        println!("Stack: {}", stack_holder.stack());

        // OP_TOALTSTACK
        OP_TOALTSTACK::execute(&mut stack_holder)?;

        // Print the stack.
        println!("Stack: {}", stack_holder.stack());

        // OP_FROMALTSTACK
        OP_FROMALTSTACK::execute(&mut stack_holder)?;

        // Print the stack.
        println!("Stack: {}", stack_holder.stack());

        // OP_FROMALTSTACK
        OP_FROMALTSTACK::execute(&mut stack_holder)?;

        // Print the stack.
        println!("Stack: {}", stack_holder.stack());

        // OP_CAT
        OP_CAT::execute(&mut stack_holder)?;

        // Print the stack.
        println!("Stack: {}", stack_holder.stack());

        // OP_EQUALVERIFY
        OP_EQUALVERIFY::execute(&mut stack_holder)?;

        // Print the stack.
        println!("Stack: {}", stack_holder.stack());

        Ok(())
    }

    #[test]
    fn arithmetic_addition_test() -> Result<(), StackError> {
        let mut stack_holder = StackHolder::new([0; 32], [0; 32], 50, 0);

        // Test 0 + 1 = 1;
        {
            // Push 1
            let item = StackItem::from_uint(StackUint::from(1));
            assert_eq!(item.bytes(), vec![0x01]);
            let _ = stack_holder.push(item);

            // Push 1
            let item = StackItem::from_uint(StackUint::from(1));
            assert_eq!(item.bytes(), vec![0x01]);
            let _ = stack_holder.push(item);

            // Push 0
            let item = StackItem::from_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // OP_ADD
            OP_ADD::execute(&mut stack_holder)?;

            // OP_VERIFY to check if addition result is equal to 0.
            OP_VERIFY::execute(&mut stack_holder)?;

            // OP_EQUALVERIFY to check if addition result is equal to 0.
            OP_EQUALVERIFY::execute(&mut stack_holder)?;

            // Stack must be empty.
            assert_eq!(stack_holder.stack_len(), 0);
        }

        // Test 0 + 0 = 0;
        {
            // Push 0
            let item = StackItem::from_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // Push 0
            let item = StackItem::from_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // Push 0
            let item = StackItem::from_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // OP_ADD
            OP_ADD::execute(&mut stack_holder)?;

            // OP_VERIFY to check if addition result is equal to 0.
            OP_VERIFY::execute(&mut stack_holder)?;

            // OP_EQUALVERIFY to check if addition result is equal to 0.
            OP_EQUALVERIFY::execute(&mut stack_holder)?;

            // Stack must be empty.
            assert_eq!(stack_holder.stack_len(), 0);
        }

        // Test 100 + 50 = 150;
        {
            // Push 150
            let item = StackItem::from_uint(StackUint::from(150));
            assert_eq!(item.bytes(), vec![0x96]);
            let _ = stack_holder.push(item);

            // Push 100
            let item = StackItem::from_uint(StackUint::from(100));
            assert_eq!(item.bytes(), vec![0x64]);
            let _ = stack_holder.push(item);

            // Push 50
            let item = StackItem::from_uint(StackUint::from(50));
            assert_eq!(item.bytes(), vec![0x32]);
            let _ = stack_holder.push(item);

            // OP_ADD
            OP_ADD::execute(&mut stack_holder)?;

            // OP_VERIFY to check if additon was successful.
            OP_VERIFY::execute(&mut stack_holder)?;

            // OP_EQUALVERIFY to check if addition result is equal to 150.
            OP_EQUALVERIFY::execute(&mut stack_holder)?;

            // Stack must be empty.
            assert_eq!(stack_holder.stack_len(), 0);
        }

        Ok(())
    }

    #[test]
    fn flow_test() -> Result<(), StackError> {
        let mut stack_holder = StackHolder::new([0; 32], [0; 32], 50, 0);

        // Initialize stack with true.
        let item = StackItem::true_item();
        let _ = stack_holder.push(item);

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_2
        OP_2::execute(&mut stack_holder)?;

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_3
        OP_3::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // Expected stack after execution ends with 2 on top.
        let expected_stack = Stack::new_with_items(vec![StackItem::from_uint(StackUint::from(2))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        Ok(())
    }

    #[test]
    fn nested_flow_test() -> Result<(), StackError> {
        let mut stack_holder = StackHolder::new([0; 32], [0; 32], 50, 0);

        // Initialize stack with true.
        let item = StackItem::true_item();
        let _ = stack_holder.push(item);

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_FALSE
        OP_FALSE::execute(&mut stack_holder)?;

        // Nested OP_IF/OP_ELSE/OP_ENDIF

        {
            // OP_IF
            OP_IF::execute(&mut stack_holder)?;

            // OP_2
            OP_2::execute(&mut stack_holder)?;

            // OP_ELSE
            OP_ELSE::execute(&mut stack_holder)?;

            // OP_3
            OP_3::execute(&mut stack_holder)?;

            // OP_ENDIF
            OP_ENDIF::execute(&mut stack_holder)?;
        }

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_4
        OP_4::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // Print the stack.
        println!("Stack 4: {}", stack_holder.stack());

        Ok(())
    }
}

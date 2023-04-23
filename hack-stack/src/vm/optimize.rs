use super::{ast::PushInstruction, ir::Instruction};
use crate::{
    common::Span,
    vm::{ast, ir::ExtInst},
};

pub fn optimize_const_binary_ops(insts: Vec<Instruction>) -> Vec<Instruction> {
    use ast::Instruction::*;
    use Instruction::Vm;

    let mut new_insts = Vec::with_capacity(insts.len());
    for inst in insts {
        if !matches!(inst, Vm(Add(_) | Sub(_))) {
            new_insts.push(inst);
            continue;
        }

        let inst2 = new_insts.pop();
        let inst1 = new_insts.pop();
        let (push1, push2) = match (inst1, inst2) {
            (Some(Vm(Push(p1))), Some(Vm(Push(p2)))) => (p1, p2),
            (inst1, inst2) => {
                if let Some(op) = inst1 {
                    new_insts.push(op);
                }
                if let Some(op) = inst2 {
                    new_insts.push(op)
                }
                new_insts.push(inst);
                continue;
            }
        };

        new_insts.extend(match inst {
            Vm(Add(span)) => optimize_const_binary_add(push1, push2, span),
            Vm(Sub(span)) => optimize_const_binary_sub(push1, push2, span),
            inst => vec![Vm(Push(push1)), Vm(Push(push2)), inst],
        });
    }
    new_insts
}

const MAX_U15: u16 = 0x7fff;

fn optimize_const_binary_add<'a>(
    push1: PushInstruction,
    push2: PushInstruction,
    add_span: Span,
) -> Vec<Instruction<'a>> {
    use ast::Instruction::*;
    use ast::Segment::*;
    use Instruction::{Ext, Vm};

    match (push1.segment, push2.segment) {
        // push const; push const; add => push (const + const)
        (Constant, Constant) => match push1.offset.checked_add(push2.offset) {
            Some(sum) if sum <= MAX_U15 => {
                vec![Vm(Push(PushInstruction {
                    segment: Constant,
                    offset: sum,
                    span: push1.span.merge(&push2.span).merge(&add_span),
                }))]
            }
            _ => {
                vec![Vm(Push(push1)), Vm(Push(push2)), Vm(Add(add_span))]
            }
        },
        // push const; push var; add => push var; add_const
        (Constant, _) => {
            vec![Vm(Push(push2)), Ext(ExtInst::AddConst(push1.offset))]
        }
        // push var; push const; add => push var; add_const
        (_, Constant) => {
            vec![Vm(Push(push1)), Ext(ExtInst::AddConst(push2.offset))]
        }
        (_, _) => {
            vec![Vm(Push(push1)), Vm(Push(push2)), Vm(Add(add_span))]
        }
    }
}

fn optimize_const_binary_sub<'a>(
    push1: PushInstruction,
    push2: PushInstruction,
    sub_span: Span,
) -> Vec<Instruction<'a>> {
    use ast::Instruction::*;
    use ast::Segment::*;
    use Instruction::{Ext, Vm};
    match (push1.segment, push2.segment) {
        // push const; push const; sub => push (const - const)
        (Constant, Constant) => match push1.offset.checked_sub(push2.offset) {
            Some(result) if result <= MAX_U15 => {
                vec![Vm(Push(PushInstruction {
                    segment: Constant,
                    offset: result,
                    span: push1.span.merge(&push2.span).merge(&sub_span),
                }))]
            }
            _ => {
                vec![Vm(Push(push1)), Vm(Push(push2)), Vm(Add(sub_span))]
            }
        },
        // push const; push var; add => push var; neg; add_const
        (Constant, _) => {
            vec![
                Vm(Push(push2)),
                Vm(Neg(push1.span)),
                Ext(ExtInst::AddConst(push1.offset)),
            ]
        }
        // push var; push const; sub => push var; sub_const
        (_, Constant) => {
            vec![Vm(Push(push1)), Ext(ExtInst::SubConst(push2.offset))]
        }
        (_, _) => {
            vec![Vm(Push(push1)), Vm(Push(push2)), Vm(Sub(sub_span))]
        }
    }
}

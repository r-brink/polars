//! Optimization that collapses several a join with several filters into faster join.
//!
//! For example, `join(how='cross').filter(pl.col.l == pl.col.r)` can be collapsed to
//! `join(how='inner', left_on=pl.col.l, right_on=pl.col.r)`.

use std::sync::Arc;

use polars_core::schema::*;
#[cfg(feature = "iejoin")]
use polars_ops::frame::{IEJoinOptions, InequalityOperator};
use polars_ops::frame::{JoinCoalesce, JoinType, MaintainOrderJoin};
use polars_utils::arena::{Arena, Node};

use super::{AExpr, ExprOrigin, IR, JoinOptionsIR, aexpr_to_leaf_names_iter};
use crate::dsl::{JoinTypeOptionsIR, Operator};
use crate::plans::optimizer::join_utils::remove_suffix;
use crate::plans::{ExprIR, MintermIter};

fn and_expr(left: Node, right: Node, expr_arena: &mut Arena<AExpr>) -> Node {
    expr_arena.add(AExpr::BinaryExpr {
        left,
        op: Operator::And,
        right,
    })
}

pub fn optimize(
    root: Node,
    lp_arena: &mut Arena<IR>,
    expr_arena: &mut Arena<AExpr>,
    streaming: bool,
) {
    let mut predicates = Vec::with_capacity(4);

    // Partition to:
    // - equality predicates
    // - IEjoin supported inequality predicates
    // - remaining predicates
    #[cfg(feature = "iejoin")]
    let mut ie_op = Vec::new();
    let mut remaining_predicates = Vec::new();

    let mut ir_stack = Vec::with_capacity(16);
    ir_stack.push(root);

    while let Some(current) = ir_stack.pop() {
        let current_ir = lp_arena.get(current);
        current_ir.copy_inputs(&mut ir_stack);

        match current_ir {
            IR::Filter {
                input: _,
                predicate,
            } => {
                predicates.push((current, predicate.node()));
            },
            IR::Join {
                input_left,
                input_right,
                schema,
                left_on,
                right_on,
                options,
            } if options.args.how.is_cross() => {
                if predicates.is_empty() {
                    continue;
                }

                let suffix = options.args.suffix();

                debug_assert!(left_on.is_empty());
                debug_assert!(right_on.is_empty());

                let mut eq_left_on = Vec::new();
                let mut eq_right_on = Vec::new();

                #[cfg(feature = "iejoin")]
                let mut ie_left_on = Vec::new();
                #[cfg(feature = "iejoin")]
                let mut ie_right_on = Vec::new();

                #[cfg(feature = "iejoin")]
                {
                    ie_op.clear();
                }

                remaining_predicates.clear();

                #[cfg(feature = "iejoin")]
                fn to_inequality_operator(op: &Operator) -> Option<InequalityOperator> {
                    match op {
                        Operator::Lt => Some(InequalityOperator::Lt),
                        Operator::LtEq => Some(InequalityOperator::LtEq),
                        Operator::Gt => Some(InequalityOperator::Gt),
                        Operator::GtEq => Some(InequalityOperator::GtEq),
                        _ => None,
                    }
                }

                let left_schema = lp_arena.get(*input_left).schema(lp_arena);
                let right_schema = lp_arena.get(*input_right).schema(lp_arena);

                let left_schema = left_schema.as_ref();
                let right_schema = right_schema.as_ref();

                for (_, predicate_node) in &predicates {
                    for node in MintermIter::new(*predicate_node, expr_arena) {
                        let AExpr::BinaryExpr { left, op, right } = expr_arena.get(node) else {
                            remaining_predicates.push(node);
                            continue;
                        };

                        if !op.is_comparison_or_bitwise() {
                            // @NOTE: This is not a valid predicate, but we should not handle that
                            // here.
                            remaining_predicates.push(node);
                            continue;
                        }

                        let mut left = *left;
                        let mut op = *op;
                        let mut right = *right;

                        let left_origin = ExprOrigin::get_expr_origin(
                            left,
                            expr_arena,
                            left_schema,
                            right_schema,
                            suffix.as_str(),
                            None,
                        )
                        .unwrap();
                        let right_origin = ExprOrigin::get_expr_origin(
                            right,
                            expr_arena,
                            left_schema,
                            right_schema,
                            suffix.as_str(),
                            None,
                        )
                        .unwrap();

                        use ExprOrigin as EO;

                        // We can only join if both sides of the binary expression stem from
                        // different sides of the join.
                        match (left_origin, right_origin) {
                            (EO::Both, _) | (_, EO::Both) => {
                                // If either expression originates from the both sides, we need to
                                // filter it afterwards.
                                remaining_predicates.push(node);
                                continue;
                            },
                            (EO::None, _) | (_, EO::None) => {
                                // @TODO: This should probably be pushed down
                                remaining_predicates.push(node);
                                continue;
                            },
                            (EO::Left, EO::Left) | (EO::Right, EO::Right) => {
                                // @TODO: This can probably be pushed down in the predicate
                                // pushdown, but for now just take it as is.
                                remaining_predicates.push(node);
                                continue;
                            },
                            (EO::Right, EO::Left) => {
                                // Swap around the expressions so they match with the left_on and
                                // right_on.
                                std::mem::swap(&mut left, &mut right);
                                op = op.swap_operands();
                            },
                            (EO::Left, EO::Right) => {},
                        }

                        if matches!(op, Operator::Eq) {
                            eq_left_on.push(ExprIR::from_node(left, expr_arena));
                            eq_right_on.push(ExprIR::from_node(right, expr_arena));
                        } else {
                            #[cfg(feature = "iejoin")]
                            if let Some(ie_op_) = to_inequality_operator(&op) {
                                fn is_numeric(
                                    node: Node,
                                    expr_arena: &Arena<AExpr>,
                                    schema: &Schema,
                                ) -> bool {
                                    aexpr_to_leaf_names_iter(node, expr_arena).any(|name| {
                                        if let Some(dt) = schema.get(name.as_str()) {
                                            dt.to_physical().is_primitive_numeric()
                                        } else {
                                            false
                                        }
                                    })
                                }

                                // We fallback to remaining if:
                                // - we already have an IEjoin or Inner join
                                // - we already have an Inner join
                                // - data is not numeric (our iejoin doesn't yet implement that)
                                if ie_op.len() >= 2
                                    || !eq_left_on.is_empty()
                                    || !is_numeric(left, expr_arena, left_schema)
                                {
                                    remaining_predicates.push(node);
                                } else {
                                    ie_left_on.push(ExprIR::from_node(left, expr_arena));
                                    ie_right_on.push(ExprIR::from_node(right, expr_arena));
                                    ie_op.push(ie_op_);
                                }
                            } else {
                                remaining_predicates.push(node);
                            }

                            #[cfg(not(feature = "iejoin"))]
                            remaining_predicates.push(node);
                        }
                    }
                }

                let mut can_simplify_join = false;

                if !eq_left_on.is_empty() {
                    for expr in eq_right_on.iter_mut() {
                        remove_suffix(expr, expr_arena, right_schema, suffix.as_str());
                    }
                    can_simplify_join = true;
                } else {
                    #[cfg(feature = "iejoin")]
                    if !ie_op.is_empty() {
                        for expr in ie_right_on.iter_mut() {
                            remove_suffix(expr, expr_arena, right_schema, suffix.as_str());
                        }
                        can_simplify_join = true;
                    }
                    can_simplify_join |= options.args.how.is_cross();
                }

                if can_simplify_join {
                    let new_join = insert_fitting_join(
                        eq_left_on,
                        eq_right_on,
                        #[cfg(feature = "iejoin")]
                        ie_left_on,
                        #[cfg(feature = "iejoin")]
                        ie_right_on,
                        #[cfg(feature = "iejoin")]
                        &ie_op,
                        &remaining_predicates,
                        lp_arena,
                        expr_arena,
                        options.as_ref().clone(),
                        *input_left,
                        *input_right,
                        schema.clone(),
                        streaming,
                    );

                    lp_arena.swap(predicates[0].0, new_join);
                }

                predicates.clear();
            },
            _ => {
                predicates.clear();
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn insert_fitting_join(
    eq_left_on: Vec<ExprIR>,
    eq_right_on: Vec<ExprIR>,
    #[cfg(feature = "iejoin")] ie_left_on: Vec<ExprIR>,
    #[cfg(feature = "iejoin")] ie_right_on: Vec<ExprIR>,
    #[cfg(feature = "iejoin")] ie_op: &[InequalityOperator],
    remaining_predicates: &[Node],
    lp_arena: &mut Arena<IR>,
    expr_arena: &mut Arena<AExpr>,
    mut options: JoinOptionsIR,
    input_left: Node,
    input_right: Node,
    schema: SchemaRef,
    streaming: bool,
) -> Node {
    debug_assert_eq!(eq_left_on.len(), eq_right_on.len());
    #[cfg(feature = "iejoin")]
    {
        debug_assert_eq!(ie_op.len(), ie_left_on.len());
        debug_assert_eq!(ie_left_on.len(), ie_right_on.len());
        debug_assert!(ie_op.len() <= 2);
    }
    debug_assert!(matches!(options.args.how, JoinType::Cross));

    let remaining_predicates = remaining_predicates
        .iter()
        .copied()
        .reduce(|left, right| and_expr(left, right, expr_arena));

    let (left_on, right_on, remaining_predicates) = match () {
        _ if !eq_left_on.is_empty() => {
            options.args.how = JoinType::Inner;
            // We need to make sure not to delete any columns
            options.args.coalesce = JoinCoalesce::KeepColumns;

            #[cfg(feature = "iejoin")]
            let remaining_predicates = ie_left_on.into_iter().zip(ie_op).zip(ie_right_on).fold(
                remaining_predicates,
                |acc, ((left, op), right)| {
                    let e = expr_arena.add(AExpr::BinaryExpr {
                        left: left.node(),
                        op: (*op).into(),
                        right: right.node(),
                    });
                    Some(acc.map_or(e, |acc| and_expr(acc, e, expr_arena)))
                },
            );

            (eq_left_on, eq_right_on, remaining_predicates)
        },
        #[cfg(feature = "iejoin")]
        _ if !ie_op.is_empty() => {
            // We can only IE join up to 2 operators

            let operator1 = ie_op[0];
            let operator2 = ie_op.get(1).copied();

            // Do an IEjoin.
            options.args.how = JoinType::IEJoin;
            options.options = Some(JoinTypeOptionsIR::IEJoin(IEJoinOptions {
                operator1,
                operator2,
            }));
            // We need to make sure not to delete any columns
            options.args.coalesce = JoinCoalesce::KeepColumns;

            (ie_left_on, ie_right_on, remaining_predicates)
        },
        // If anything just fall back to a cross join.
        _ => {
            options.args.how = JoinType::Cross;
            // We need to make sure not to delete any columns
            options.args.coalesce = JoinCoalesce::KeepColumns;

            #[cfg(feature = "iejoin")]
            let remaining_predicates = ie_left_on.into_iter().zip(ie_op).zip(ie_right_on).fold(
                remaining_predicates,
                |acc, ((left, op), right)| {
                    let e = expr_arena.add(AExpr::BinaryExpr {
                        left: left.node(),
                        op: (*op).into(),
                        right: right.node(),
                    });
                    Some(acc.map_or(e, |acc| and_expr(acc, e, expr_arena)))
                },
            );

            let mut remaining_predicates = remaining_predicates;
            if let Some(pred) = remaining_predicates.take_if(|_| {
                matches!(options.args.maintain_order, MaintainOrderJoin::None) && !streaming
            }) {
                options.options = Some(JoinTypeOptionsIR::Cross {
                    predicate: ExprIR::from_node(pred, expr_arena),
                })
            }

            (Vec::new(), Vec::new(), remaining_predicates)
        },
    };

    // Note: We expect key type upcasting / expression optimizations have already been done during
    // DSL->IR conversion.

    let join_ir = IR::Join {
        input_left,
        input_right,
        schema,
        left_on,
        right_on,
        options: Arc::new(options),
    };

    let join_node = lp_arena.add(join_ir);

    if let Some(predicate) = remaining_predicates {
        lp_arena.add(IR::Filter {
            input: join_node,
            predicate: ExprIR::from_node(predicate, &*expr_arena),
        })
    } else {
        join_node
    }
}

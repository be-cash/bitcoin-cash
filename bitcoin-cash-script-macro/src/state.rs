use crate::ir;
use bitcoin_cash_base::Integer;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;

#[derive(Clone)]
pub struct StackItem {
    pub ident: syn::Ident,
    pub name: String,
    pub has_generated_name: bool,
    pub integer: Option<Integer>,
}

#[derive(Clone)]
pub struct State {
    pub condition: ir::VariantPredicate,
    pub stack: Vec<StackItem>,
    pub alt_stack: Vec<StackItem>,
}

#[derive(Clone)]
pub struct VariantStates {
    pub states: HashMap<syn::Ident, State>,
    pub predicate_atoms: Vec<ir::VariantPredicateAtom>,
}

impl StackItem {
    pub fn name_tokens(&self) -> TokenStream {
        if self.has_generated_name {
            quote! { None }
        } else {
            let name = &self.name;
            quote! { Some(#name.into()) }
        }
    }
}

impl VariantStates {
    pub fn push(&mut self, stack_item: StackItem) {
        for state in self.states.values_mut() {
            if state.condition.holds(&self.predicate_atoms) {
                state.stack.push(stack_item.clone());
            }
        }
    }

    pub fn push_alt(&mut self, stack_item: StackItem) {
        for state in self.states.values_mut() {
            if state.condition.holds(&self.predicate_atoms) {
                state.alt_stack.push(stack_item.clone());
            }
        }
    }

    fn pick_roll(&mut self, item_depth: usize, is_roll: bool) -> Result<StackItem, String> {
        let mut prev_item: Option<StackItem> = None;
        let mut prev_variant = None;
        for (next_variant, next_state) in self.states.iter_mut() {
            if next_state.condition.holds(&self.predicate_atoms) {
                let stack = &mut next_state.stack;
                if item_depth >= stack.len() {
                    return Err(format!(
                        "Tried accessing {} items deep, but stack only has {} items",
                        item_depth,
                        stack.len(),
                    ));
                }
                let item_idx = stack.len() - item_depth - 1;
                let next_item = if is_roll {
                    stack.remove(item_idx)
                } else {
                    stack[item_idx].clone()
                };
                let opcode_name = if is_roll { "OP_ROLL" } else { "OP_PICK" };
                if let (Some(prev_item), Some(prev_variant)) = (&prev_item, &prev_variant) {
                    if next_item.has_generated_name != prev_item.has_generated_name {
                        return Err(format!(
                            "{} {} results in inconsistent stack item names. \
                                Item in variant {} {} a name{} while in \
                                variant {} it {}{}.",
                            item_depth,
                            opcode_name,
                            prev_variant,
                            if !prev_item.has_generated_name {
                                "has"
                            } else {
                                "doesn't have"
                            },
                            if prev_item.has_generated_name {
                                String::default()
                            } else {
                                format!(" ({})", prev_item.name)
                            },
                            next_variant,
                            if !next_item.has_generated_name {
                                "does"
                            } else {
                                "doesn't"
                            },
                            if next_item.has_generated_name {
                                String::default()
                            } else {
                                format!(" ({})", next_item.name)
                            },
                        ));
                    } else if !next_item.has_generated_name
                        && !prev_item.has_generated_name
                        && next_item.name != prev_item.name
                    {
                        return Err(format!(
                            "Branch results in inconsistent stack item names. \
                                    Item in variant `{}` is named `{}` while \
                                    in variant `{}` is named `{}` in .",
                            prev_variant, prev_item.name, next_variant, next_item.name,
                        ));
                    }
                }
                prev_item = Some(next_item);
                prev_variant = Some(next_variant);
            }
        }
        prev_item.ok_or_else(|| "No variant for this op".to_string())
    }

    pub fn find_item(&mut self, ident: &syn::Ident) -> Result<(usize, &StackItem), String> {
        let mut prev_depth_item: Option<(usize, &StackItem)> = None;
        let mut prev_stack: Option<&[StackItem]> = None;
        let mut prev_variant = None;
        for (next_variant, next_state) in self.states.iter_mut() {
            if next_state.condition.holds(&self.predicate_atoms) {
                let stack = &mut next_state.stack;
                let next_depth = stack
                    .iter()
                    .rev()
                    .position(|stack_item| stack_item.name == ident.to_string())
                    .ok_or_else(|| {
                        format!("Couldn't find {} in variant {}", ident, next_variant)
                    })?;
                let next_idx = stack.len() - next_depth - 1;
                if let (Some((depth, _)), Some(prev_variant), Some(prev_stack)) =
                    (prev_depth_item, &prev_variant, &prev_stack)
                {
                    if depth != next_depth {
                        return Err(format!(
                            "Inconsistent item depths. Item `{}` in variant `{}` is {} \
                             items deep, but the same item in variant `{}` is {} items deep. \n \
                             Stack in variant `{}`: {}\n \
                             Stack in variant `{}`: {}\n \
                             predicates: {}",
                            ident,
                            prev_variant,
                            depth,
                            next_variant,
                            next_depth,
                            prev_variant,
                            prev_stack
                                .iter()
                                .map(|item| item.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            next_variant,
                            stack
                                .iter()
                                .map(|item| item.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            self.predicate_atoms
                                .iter()
                                .map(|predicate| predicate.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ));
                    }
                }
                prev_depth_item = Some((next_depth, &stack[next_idx]));
                prev_variant = Some(next_variant);
                prev_stack = Some(stack);
            }
        }
        prev_depth_item.ok_or_else(|| "No variant for this op".to_string())
    }

    fn pop_flagged(&mut self, is_alt_stack: bool) -> Result<StackItem, String> {
        let mut prev_item: Option<StackItem> = None;
        let mut prev_variant = None;
        for (next_variant, stack) in self.states.iter_mut() {
            if stack.condition.holds(&self.predicate_atoms) {
                let next_item = if is_alt_stack {
                    stack.alt_stack.pop()
                } else {
                    stack.stack.pop()
                }
                .ok_or(format!("Empty stack on variant {}", next_variant))?;
                if let (Some(prev_item), Some(prev_variant)) = (&prev_item, &prev_variant) {
                    if next_item.has_generated_name != prev_item.has_generated_name {
                        return Err(format!(
                            "Pop results in inconsistent stack item names. \
                                Top stack item in variant {} {} a name{} while in \
                                variant {} it {}{}.",
                            prev_variant,
                            if !prev_item.has_generated_name {
                                "has"
                            } else {
                                "doesn't have"
                            },
                            if prev_item.has_generated_name {
                                String::default()
                            } else {
                                format!(" ({})", prev_item.name)
                            },
                            next_variant,
                            if !next_item.has_generated_name {
                                "does"
                            } else {
                                "doesn't"
                            },
                            if next_item.has_generated_name {
                                String::default()
                            } else {
                                format!(" ({})", next_item.name)
                            },
                        ));
                    } else if !next_item.has_generated_name
                        && !prev_item.has_generated_name
                        && next_item.name != prev_item.name
                    {
                        return Err(format!(
                            "Branch results in inconsistent stack item names. \
                                    Top item in variant `{}` is named `{}` while \
                                    in variant `{}` is named `{}` in .",
                            prev_variant, prev_item.name, next_variant, next_item.name,
                        ));
                    }
                }
                prev_item = Some(next_item);
                prev_variant = Some(next_variant);
            }
        }
        prev_item.ok_or(format!(
            "No variant for this op (predicate stack is {})",
            self.predicate_atoms
                .iter()
                .map(|predicate| predicate.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ))
    }

    pub fn pop(&mut self) -> Result<StackItem, String> {
        self.pop_flagged(false)
    }

    pub fn pop_alt(&mut self) -> Result<StackItem, String> {
        self.pop_flagged(true)
    }

    pub fn pick(&mut self, item_depth: usize) -> Result<StackItem, String> {
        self.pick_roll(item_depth, false)
    }

    pub fn roll(&mut self, item_depth: usize) -> Result<StackItem, String> {
        self.pick_roll(item_depth, true)
    }
}

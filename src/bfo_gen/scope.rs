use super::BFOGenerator;
use crate::hbf_ast::Expr;

impl BFOGenerator {
    pub(super) fn push_scope(&mut self) {
        self.variables.push(std::collections::HashMap::new());
    }

    pub(super) fn pop_scope(&mut self) {
        if self.variables.len() > 1 {
            self.variables.pop();
        }
    }

    pub(super) fn get_variable(&self, name: &str) -> Option<Expr> {
        for scope in self.variables.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    pub(super) fn declare_variable(&mut self, name: &str, val: Expr) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), val);
        }
    }

    pub(super) fn set_variable(&mut self, name: &str, val: Expr) {
        for scope in self.variables.iter_mut().rev() {
            if let Some(existing_val) = scope.get_mut(name) {
                *existing_val = val.clone();
                return;
            }
        }
        panic!("Variable {} not found", name);
    }

    pub(super) fn get_array_var_name(&self, name: &str, index: i32) -> String {
        if let Some((_, _, elem_type, _)) = self.arrays.get(name) {
            if elem_type.is_virtual() {
                panic!("Array {} has virtual elements, cannot get cell name", name);
            }
        }
        format!("__hbf_cell_{}_{}", name, index)
    }
}

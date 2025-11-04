use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Expression {
    Literal(Literal),
    Var(String),
    Lambda {
        param: String,
        body: Box<Expression>,
    },
    Apply {
        func: Box<Expression>,
        arg: Box<Expression>,
    },
    LinearApply {
        func: Box<Expression>,
        arg: Box<Expression>,
    },
    Let {
        name: String,
        value: Box<Expression>,
        body: Box<Expression>,
    },
    Match {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    Tuple(Vec<Expression>),
    List(Vec<Expression>),
    Record(Vec<(String, Expression)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Literal {
    Int(i64),
    Float(String),
    String(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expression>>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Pattern {
    Wildcard,
    Var(String),
    Literal(Literal),
    Bind {
        name: String,
        pattern: Box<Pattern>,
    },
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),
    Constructor {
        name: String,
        args: Vec<Pattern>,
    },
    Record(Vec<(String, Pattern)>),
    Lambda {
        param_pattern: Box<Pattern>,
        body_pattern: Box<Pattern>,
    },
    Apply {
        func_pattern: Box<Pattern>,
        arg_pattern: Box<Pattern>,
    },
}

pub type Bindings = HashMap<String, Expression>;
pub type MatchResult = Vec<Bindings>;

pub fn match_pattern(expr: &Expression, pattern: &Pattern) -> MatchResult {
    let mut bindings = HashMap::new();
    if match_pattern_internal(expr, pattern, &mut bindings) {
        vec![bindings]
    } else {
        vec![]
    }
}

fn match_pattern_internal(
    expr: &Expression,
    pattern: &Pattern,
    bindings: &mut Bindings,
) -> bool {
    match pattern {
        Pattern::Wildcard => true,
        
        Pattern::Var(name) => {
            if let Some(existing) = bindings.get(name) {
                existing == expr
            } else {
                bindings.insert(name.clone(), expr.clone());
                true
            }
        }
        
        Pattern::Literal(pat_lit) => {
            matches!(expr, Expression::Literal(expr_lit) if expr_lit == pat_lit)
        }
        
        Pattern::Bind { name, pattern: inner_pattern } => {
            if match_pattern_internal(expr, inner_pattern, bindings) {
                if let Some(existing) = bindings.get(name) {
                    existing == expr
                } else {
                    bindings.insert(name.clone(), expr.clone());
                    true
                }
            } else {
                false
            }
        }
        
        Pattern::Tuple(pat_elements) => {
            if let Expression::Tuple(expr_elements) = expr {
                if pat_elements.len() != expr_elements.len() {
                    return false;
                }
                
                for (expr_elem, pat_elem) in expr_elements.iter().zip(pat_elements.iter()) {
                    if !match_pattern_internal(expr_elem, pat_elem, bindings) {
                        return false;
                    }
                }
                true
            } else {
                false
            }
        }
        
        Pattern::List(pat_elements) => {
            if let Expression::List(expr_elements) = expr {
                if pat_elements.len() != expr_elements.len() {
                    return false;
                }
                
                for (expr_elem, pat_elem) in expr_elements.iter().zip(pat_elements.iter()) {
                    if !match_pattern_internal(expr_elem, pat_elem, bindings) {
                        return false;
                    }
                }
                true
            } else {
                false
            }
        }
        
        Pattern::Constructor { name, args } => {
            match_constructor(expr, name, args, bindings)
        }
        
        Pattern::Record(pat_fields) => {
            if let Expression::Record(expr_fields) = expr {
                for (pat_key, pat_val_pattern) in pat_fields {
                    match expr_fields.iter().find(|(k, _)| k == pat_key) {
                        Some((_, expr_val)) => {
                            if !match_pattern_internal(expr_val, pat_val_pattern, bindings) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            } else {
                false
            }
        }
        
        Pattern::Lambda { param_pattern, body_pattern } => {
            if let Expression::Lambda { param, body } = expr {
                let param_expr = Expression::Var(param.clone());
                if !match_pattern_internal(&param_expr, param_pattern, bindings) {
                    return false;
                }
                
                match_pattern_internal(body, body_pattern, bindings)
            } else {
                false
            }
        }
        
        Pattern::Apply { func_pattern, arg_pattern } => {
            match expr {
                Expression::Apply { func, arg } | Expression::LinearApply { func, arg } => {
                    match_pattern_internal(func, func_pattern, bindings)
                        && match_pattern_internal(arg, arg_pattern, bindings)
                }
                _ => false,
            }
        }
    }
}

fn match_constructor(
    expr: &Expression,
    name: &str,
    args: &[Pattern],
    bindings: &mut Bindings,
) -> bool {
    if args.is_empty() {
        matches!(expr, Expression::Var(v) if v == name)
    } else {
        match_constructor_application(expr, name, args, bindings)
    }
}

fn match_constructor_application(
    expr: &Expression,
    name: &str,
    args: &[Pattern],
    bindings: &mut Bindings,
) -> bool {
    if args.len() == 1 {
        if let Expression::Apply { func, arg } = expr {
            if let Expression::Var(func_name) = func.as_ref() {
                if func_name == name {
                    return match_pattern_internal(arg, &args[0], bindings);
                }
            }
        }
    } else if args.len() > 1 {
        let mut current = expr;
        let mut matched_args = Vec::new();
        
        while let Expression::Apply { func, arg } = current {
            matched_args.push(arg.as_ref());
            current = func.as_ref();
        }
        
        if let Expression::Var(func_name) = current {
            if func_name == name && matched_args.len() == args.len() {
                matched_args.reverse();
                
                for (expr_arg, pat_arg) in matched_args.iter().zip(args.iter()) {
                    if !match_pattern_internal(expr_arg, pat_arg, bindings) {
                        return false;
                    }
                }
                return true;
            }
        }
    }
    
    false
}

pub fn match_any_pattern(expr: &Expression, patterns: &[Pattern]) -> MatchResult {
    let mut all_results = Vec::new();
    
    for pattern in patterns {
        let results = match_pattern(expr, pattern);
        all_results.extend(results);
    }
    
    all_results
}

pub fn match_pattern_many(exprs: &[Expression], pattern: &Pattern) -> Vec<MatchResult> {
    exprs.iter().map(|expr| match_pattern(expr, pattern)).collect()
}

pub fn matches(expr: &Expression, pattern: &Pattern) -> bool {
    !match_pattern(expr, pattern).is_empty()
}

pub fn pattern_variables(pattern: &Pattern) -> Vec<String> {
    let mut vars = Vec::new();
    collect_pattern_variables(pattern, &mut vars);
    vars.sort();
    vars.dedup();
    vars
}

fn collect_pattern_variables(pattern: &Pattern, vars: &mut Vec<String>) {
    match pattern {
        Pattern::Wildcard | Pattern::Literal(_) => {}
        Pattern::Var(name) => vars.push(name.clone()),
        Pattern::Bind { name, pattern } => {
            vars.push(name.clone());
            collect_pattern_variables(pattern, vars);
        }
        Pattern::Tuple(patterns) | Pattern::List(patterns) => {
            for p in patterns {
                collect_pattern_variables(p, vars);
            }
        }
        Pattern::Constructor { args, .. } => {
            for p in args {
                collect_pattern_variables(p, vars);
            }
        }
        Pattern::Record(fields) => {
            for (_, p) in fields {
                collect_pattern_variables(p, vars);
            }
        }
        Pattern::Lambda { param_pattern, body_pattern } => {
            collect_pattern_variables(param_pattern, vars);
            collect_pattern_variables(body_pattern, vars);
        }
        Pattern::Apply { func_pattern, arg_pattern } => {
            collect_pattern_variables(func_pattern, vars);
            collect_pattern_variables(arg_pattern, vars);
        }
    }
}

pub fn serialize_match_result(result: &MatchResult) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    ciborium::into_writer(result, &mut buffer)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    Ok(buffer)
}

pub fn deserialize_match_result(data: &[u8]) -> Result<MatchResult, String> {
    ciborium::from_reader(data)
        .map_err(|e| format!("Deserialization failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn var(name: &str) -> Expression {
        Expression::Var(name.to_string())
    }

    fn int(n: i64) -> Expression {
        Expression::Literal(Literal::Int(n))
    }

    fn bool_lit(b: bool) -> Expression {
        Expression::Literal(Literal::Bool(b))
    }

    fn string_lit(s: &str) -> Expression {
        Expression::Literal(Literal::String(s.to_string()))
    }

    fn lambda(param: &str, body: Expression) -> Expression {
        Expression::Lambda {
            param: param.to_string(),
            body: Box::new(body),
        }
    }

    fn apply(func: Expression, arg: Expression) -> Expression {
        Expression::Apply {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }

    #[test]
    fn test_wildcard_pattern() {
        let expr = int(42);
        let pattern = Pattern::Wildcard;
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_empty());
    }

    #[test]
    fn test_variable_pattern() {
        let expr = int(42);
        let pattern = Pattern::Var("x".to_string());
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(42)));
    }

    #[test]
    fn test_literal_pattern_match() {
        let expr = int(42);
        let pattern = Pattern::Literal(Literal::Int(42));
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_literal_pattern_no_match() {
        let expr = int(42);
        let pattern = Pattern::Literal(Literal::Int(43));
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_bind_pattern() {
        let expr = int(42);
        let pattern = Pattern::Bind {
            name: "x".to_string(),
            pattern: Box::new(Pattern::Literal(Literal::Int(42))),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(42)));
    }

    #[test]
    fn test_bind_pattern_with_wildcard() {
        let expr = int(42);
        let pattern = Pattern::Bind {
            name: "x".to_string(),
            pattern: Box::new(Pattern::Wildcard),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(42)));
    }

    #[test]
    fn test_tuple_pattern() {
        let expr = Expression::Tuple(vec![int(1), int(2), int(3)]);
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("x".to_string()),
            Pattern::Var("y".to_string()),
            Pattern::Var("z".to_string()),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(1)));
        assert_eq!(results[0].get("y"), Some(&int(2)));
        assert_eq!(results[0].get("z"), Some(&int(3)));
    }

    #[test]
    fn test_tuple_pattern_wrong_length() {
        let expr = Expression::Tuple(vec![int(1), int(2)]);
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("x".to_string()),
            Pattern::Var("y".to_string()),
            Pattern::Var("z".to_string()),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list_pattern() {
        let expr = Expression::List(vec![int(1), int(2), int(3)]);
        let pattern = Pattern::List(vec![
            Pattern::Literal(Literal::Int(1)),
            Pattern::Wildcard,
            Pattern::Var("z".to_string()),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("z"), Some(&int(3)));
    }

    #[test]
    fn test_constructor_pattern_zero_args() {
        let expr = var("None");
        let pattern = Pattern::Constructor {
            name: "None".to_string(),
            args: vec![],
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_constructor_pattern_one_arg() {
        let expr = apply(var("Some"), int(42));
        let pattern = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![Pattern::Var("x".to_string())],
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(42)));
    }

    #[test]
    fn test_constructor_pattern_multiple_args() {
        let expr = apply(apply(var("Pair"), int(1)), int(2));
        let pattern = Pattern::Constructor {
            name: "Pair".to_string(),
            args: vec![
                Pattern::Var("x".to_string()),
                Pattern::Var("y".to_string()),
            ],
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(1)));
        assert_eq!(results[0].get("y"), Some(&int(2)));
    }

    #[test]
    fn test_record_pattern() {
        let expr = Expression::Record(vec![
            ("x".to_string(), int(1)),
            ("y".to_string(), int(2)),
        ]);
        
        let pattern = Pattern::Record(vec![
            ("x".to_string(), Pattern::Var("a".to_string())),
            ("y".to_string(), Pattern::Var("b".to_string())),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("a"), Some(&int(1)));
        assert_eq!(results[0].get("b"), Some(&int(2)));
    }

    #[test]
    fn test_record_pattern_partial() {
        let expr = Expression::Record(vec![
            ("x".to_string(), int(1)),
            ("y".to_string(), int(2)),
            ("z".to_string(), int(3)),
        ]);
        
        let pattern = Pattern::Record(vec![
            ("x".to_string(), Pattern::Var("a".to_string())),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("a"), Some(&int(1)));
    }

    #[test]
    fn test_lambda_pattern() {
        let expr = lambda("x", var("x"));
        let pattern = Pattern::Lambda {
            param_pattern: Box::new(Pattern::Var("p".to_string())),
            body_pattern: Box::new(Pattern::Var("b".to_string())),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("p"), Some(&var("x")));
        assert_eq!(results[0].get("b"), Some(&var("x")));
    }

    #[test]
    fn test_application_pattern() {
        let expr = apply(var("f"), int(42));
        let pattern = Pattern::Apply {
            func_pattern: Box::new(Pattern::Var("func".to_string())),
            arg_pattern: Box::new(Pattern::Var("arg".to_string())),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("func"), Some(&var("f")));
        assert_eq!(results[0].get("arg"), Some(&int(42)));
    }

    #[test]
    fn test_nested_pattern() {
        let expr = Expression::Tuple(vec![
            int(1),
            Expression::List(vec![int(2), int(3)]),
        ]);
        
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("x".to_string()),
            Pattern::List(vec![
                Pattern::Var("y".to_string()),
                Pattern::Var("z".to_string()),
            ]),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(1)));
        assert_eq!(results[0].get("y"), Some(&int(2)));
        assert_eq!(results[0].get("z"), Some(&int(3)));
    }

    #[test]
    fn test_match_any_pattern() {
        let expr = int(42);
        let patterns = vec![
            Pattern::Literal(Literal::Int(43)),
            Pattern::Var("x".to_string()),
            Pattern::Wildcard,
        ];
        
        let results = match_any_pattern(&expr, &patterns);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_match_pattern_many() {
        let exprs = vec![int(1), int(2), int(3)];
        let pattern = Pattern::Var("x".to_string());
        
        let results = match_pattern_many(&exprs, &pattern);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0][0].get("x"), Some(&int(1)));
        assert_eq!(results[1][0].get("x"), Some(&int(2)));
        assert_eq!(results[2][0].get("x"), Some(&int(3)));
    }

    #[test]
    fn test_matches_convenience() {
        let expr = int(42);
        assert!(matches(&expr, &Pattern::Wildcard));
        assert!(matches(&expr, &Pattern::Var("x".to_string())));
        assert!(matches(&expr, &Pattern::Literal(Literal::Int(42))));
        assert!(!matches(&expr, &Pattern::Literal(Literal::Int(43))));
    }

    #[test]
    fn test_pattern_variables_extraction() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("x".to_string()),
            Pattern::Bind {
                name: "y".to_string(),
                pattern: Box::new(Pattern::Var("z".to_string())),
            },
        ]);
        
        let vars = pattern_variables(&pattern);
        assert_eq!(vars, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_nested_bind_pattern() {
        let expr = Expression::Tuple(vec![int(1), int(2), int(3), int(4)]);
        
        let pattern = Pattern::Bind {
            name: "whole".to_string(),
            pattern: Box::new(Pattern::Tuple(vec![
                Pattern::Var("a".to_string()),
                Pattern::Bind {
                    name: "middle".to_string(),
                    pattern: Box::new(Pattern::Var("b".to_string())),
                },
                Pattern::Var("c".to_string()),
                Pattern::Var("d".to_string()),
            ])),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("whole"), Some(&expr));
        assert_eq!(results[0].get("a"), Some(&int(1)));
        assert_eq!(results[0].get("b"), Some(&int(2)));
        assert_eq!(results[0].get("middle"), Some(&int(2)));
        assert_eq!(results[0].get("c"), Some(&int(3)));
        assert_eq!(results[0].get("d"), Some(&int(4)));
    }

    #[test]
    fn test_lambda_body_pattern() {
        let expr = lambda("x", apply(var("f"), var("x")));
        let pattern = Pattern::Lambda {
            param_pattern: Box::new(Pattern::Wildcard),
            body_pattern: Box::new(Pattern::Apply {
                func_pattern: Box::new(Pattern::Var("func".to_string())),
                arg_pattern: Box::new(Pattern::Wildcard),
            }),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("func"), Some(&var("f")));
    }

    #[test]
    fn test_linear_application_pattern() {
        let expr = Expression::LinearApply {
            func: Box::new(var("f")),
            arg: Box::new(int(42)),
        };
        
        let pattern = Pattern::Apply {
            func_pattern: Box::new(Pattern::Var("f".to_string())),
            arg_pattern: Box::new(Pattern::Var("x".to_string())),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("f"), Some(&var("f")));
        assert_eq!(results[0].get("x"), Some(&int(42)));
    }

    #[test]
    fn test_match_expression_pattern() {
        let match_expr = Expression::Match {
            expr: Box::new(var("x")),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::Int(0)),
                    guard: None,
                    body: Box::new(int(1)),
                },
            ],
        };
        
        let pattern = Pattern::Wildcard;
        let results = match_pattern(&match_expr, &pattern);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_let_expression_pattern() {
        let let_expr = Expression::Let {
            name: "x".to_string(),
            value: Box::new(int(42)),
            body: Box::new(var("x")),
        };
        
        let pattern = Pattern::Wildcard;
        let results = match_pattern(&let_expr, &pattern);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_float_literal_pattern() {
        let expr = Expression::Literal(Literal::Float("3.14".to_string()));
        let pattern = Pattern::Literal(Literal::Float("3.14".to_string()));
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_float_literal_no_match() {
        let expr = Expression::Literal(Literal::Float("3.14".to_string()));
        let pattern = Pattern::Literal(Literal::Float("2.71".to_string()));
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_complex_constructor_tree() {
        let leaf1 = apply(var("Leaf"), int(1));
        let leaf2 = apply(var("Leaf"), int(2));
        let leaf3 = apply(var("Leaf"), int(3));
        let subtree = apply(apply(var("Node"), leaf2), leaf3);
        let tree = apply(apply(var("Node"), leaf1), subtree);
        
        let pattern = Pattern::Constructor {
            name: "Node".to_string(),
            args: vec![
                Pattern::Constructor {
                    name: "Leaf".to_string(),
                    args: vec![Pattern::Var("x".to_string())],
                },
                Pattern::Wildcard,
            ],
        };
        
        let results = match_pattern(&tree, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("x"), Some(&int(1)));
    }

    #[test]
    fn test_record_missing_field() {
        let expr = Expression::Record(vec![
            ("x".to_string(), int(1)),
        ]);
        
        let pattern = Pattern::Record(vec![
            ("x".to_string(), Pattern::Wildcard),
            ("y".to_string(), Pattern::Wildcard),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_multiple_bind_patterns() {
        let expr = Expression::Tuple(vec![int(1), int(2)]);
        
        let pattern = Pattern::Tuple(vec![
            Pattern::Bind {
                name: "first".to_string(),
                pattern: Box::new(Pattern::Var("x".to_string())),
            },
            Pattern::Bind {
                name: "second".to_string(),
                pattern: Box::new(Pattern::Var("y".to_string())),
            },
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("first"), Some(&int(1)));
        assert_eq!(results[0].get("second"), Some(&int(2)));
        assert_eq!(results[0].get("x"), Some(&int(1)));
        assert_eq!(results[0].get("y"), Some(&int(2)));
    }

    #[test]
    fn test_bind_pattern_failure() {
        let expr = int(42);
        
        let pattern = Pattern::Bind {
            name: "x".to_string(),
            pattern: Box::new(Pattern::Literal(Literal::Int(43))),
        };
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_pattern_variable_shadowing() {
        let expr = Expression::Tuple(vec![
            int(1),
            Expression::Tuple(vec![int(2)]),
        ]);
        
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("x".to_string()),
            Pattern::Tuple(vec![Pattern::Var("x".to_string())]),
        ]);
        
        let results = match_pattern(&expr, &pattern);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_deterministic_binding_order() {
        let expr = Expression::Tuple(vec![int(1), int(2), int(3)]);
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("a".to_string()),
            Pattern::Var("b".to_string()),
            Pattern::Var("c".to_string()),
        ]);
        
        let results1 = match_pattern(&expr, &pattern);
        let results2 = match_pattern(&expr, &pattern);
        
        assert_eq!(results1, results2);
        
        let s1 = serialize_match_result(&results1).unwrap();
        let s2 = serialize_match_result(&results2).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_serialization_preserves_match_set() {
        let expr = Expression::List(vec![int(1), int(2), int(3)]);
        let pattern = Pattern::List(vec![
            Pattern::Var("x".to_string()),
            Pattern::Var("y".to_string()),
            Pattern::Var("z".to_string()),
        ]);
        
        let original_results = match_pattern(&expr, &pattern);
        
        let serialized = serialize_match_result(&original_results).unwrap();
        let deserialized = deserialize_match_result(&serialized).unwrap();
        
        assert_eq!(original_results, deserialized);
        
        assert_eq!(deserialized[0].get("x"), Some(&int(1)));
        assert_eq!(deserialized[0].get("y"), Some(&int(2)));
        assert_eq!(deserialized[0].get("z"), Some(&int(3)));
    }

    #[test]
    fn test_pattern_match_idempotence() {
        let expr = Expression::Tuple(vec![int(42), bool_lit(true)]);
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("num".to_string()),
            Pattern::Var("flag".to_string()),
        ]);
        
        for _ in 0..10 {
            let results = match_pattern(&expr, &pattern);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].get("num"), Some(&int(42)));
            assert_eq!(results[0].get("flag"), Some(&bool_lit(true)));
        }
    }

    #[test]
    fn test_comprehensive_structural_matching() {
        println!("\n=== Comprehensive Structural Matching Test ===");
        
        let test1_expr = Expression::Tuple(vec![int(1), int(2)]);
        let test1_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Int(1)),
            Pattern::Wildcard,
        ]);
        assert!(matches(&test1_expr, &test1_pattern));
        println!("✓ Test 1: Simple tuple with literal and wildcard");
        
        let test2_expr = Expression::List(vec![
            Expression::List(vec![int(1)]),
            Expression::List(vec![int(2)]),
        ]);
        let test2_pattern = Pattern::List(vec![
            Pattern::List(vec![Pattern::Var("x".to_string())]),
            Pattern::Wildcard,
        ]);
        let test2_results = match_pattern(&test2_expr, &test2_pattern);
        assert_eq!(test2_results.len(), 1);
        assert_eq!(test2_results[0].get("x"), Some(&int(1)));
        println!("✓ Test 2: Nested list matching");
        
        let test3_expr = apply(
            apply(var("Cons"), int(1)),
            apply(apply(var("Cons"), int(2)), var("Nil")),
        );
        let test3_pattern = Pattern::Constructor {
            name: "Cons".to_string(),
            args: vec![
                Pattern::Var("head".to_string()),
                Pattern::Wildcard,
            ],
        };
        let test3_results = match_pattern(&test3_expr, &test3_pattern);
        assert_eq!(test3_results.len(), 1);
        assert_eq!(test3_results[0].get("head"), Some(&int(1)));
        println!("✓ Test 3: Constructor pattern (linked list)");
        
        let test4_expr = lambda("x", apply(apply(var("+"), var("x")), int(1)));
        let test4_pattern = Pattern::Lambda {
            param_pattern: Box::new(Pattern::Wildcard),
            body_pattern: Box::new(Pattern::Apply {
                func_pattern: Box::new(Pattern::Apply {
                    func_pattern: Box::new(Pattern::Var("op".to_string())),
                    arg_pattern: Box::new(Pattern::Wildcard),
                }),
                arg_pattern: Box::new(Pattern::Var("increment".to_string())),
            }),
        };
        let test4_results = match_pattern(&test4_expr, &test4_pattern);
        assert_eq!(test4_results.len(), 1);
        assert_eq!(test4_results[0].get("op"), Some(&var("+")));
        assert_eq!(test4_results[0].get("increment"), Some(&int(1)));
        println!("✓ Test 4: Lambda with nested application");
        
        let test5_expr = Expression::Record(vec![
            ("user".to_string(), Expression::Record(vec![
                ("name".to_string(), string_lit("Alice")),
                ("age".to_string(), int(30)),
            ])),
        ]);
        let test5_pattern = Pattern::Record(vec![
            ("user".to_string(), Pattern::Record(vec![
                ("name".to_string(), Pattern::Var("username".to_string())),
            ])),
        ]);
        let test5_results = match_pattern(&test5_expr, &test5_pattern);
        assert_eq!(test5_results.len(), 1);
        assert_eq!(test5_results[0].get("username"), Some(&string_lit("Alice")));
        println!("✓ Test 5: Nested record matching");
        
        let test6_expr = Expression::Tuple(vec![
            int(1),
            Expression::List(vec![int(2), int(3), int(4)]),
        ]);
        let test6_pattern = Pattern::Tuple(vec![
            Pattern::Var("first".to_string()),
            Pattern::Bind {
                name: "rest".to_string(),
                pattern: Box::new(Pattern::List(vec![
                    Pattern::Var("second".to_string()),
                    Pattern::Wildcard,
                    Pattern::Var("fourth".to_string()),
                ])),
            },
        ]);
        let test6_results = match_pattern(&test6_expr, &test6_pattern);
        assert_eq!(test6_results.len(), 1);
        assert_eq!(test6_results[0].get("first"), Some(&int(1)));
        assert_eq!(test6_results[0].get("second"), Some(&int(2)));
        assert_eq!(test6_results[0].get("fourth"), Some(&int(4)));
        assert_eq!(
            test6_results[0].get("rest"),
            Some(&Expression::List(vec![int(2), int(3), int(4)]))
        );
        println!("✓ Test 6: Complex bind with nested patterns");
        
        println!("\n=== All structural matching tests passed ===");
    }

    #[test]
    fn test_round_trip_property() {
        let test_cases = vec![
            (int(42), Pattern::Var("x".to_string())),
            (
                Expression::Tuple(vec![int(1), int(2)]),
                Pattern::Tuple(vec![
                    Pattern::Var("a".to_string()),
                    Pattern::Var("b".to_string()),
                ]),
            ),
            (
                Expression::List(vec![int(1), int(2), int(3)]),
                Pattern::List(vec![
                    Pattern::Wildcard,
                    Pattern::Var("x".to_string()),
                    Pattern::Wildcard,
                ]),
            ),
        ];
        
        for (expr, pattern) in test_cases {
            let original = match_pattern(&expr, &pattern);
            let serialized = serialize_match_result(&original).unwrap();
            let deserialized = deserialize_match_result(&serialized).unwrap();
            
            assert_eq!(original, deserialized, "Round-trip failed for pattern matching");
        }
        
        println!("✓ Round-trip property verified for all test cases");
    }

    #[test]
    fn test_all_literal_types() {
        let literals = vec![
            (Literal::Int(42), Literal::Int(42), true),
            (Literal::Int(42), Literal::Int(43), false),
            (Literal::Bool(true), Literal::Bool(true), true),
            (Literal::Bool(true), Literal::Bool(false), false),
            (
                Literal::String("hello".to_string()),
                Literal::String("hello".to_string()),
                true,
            ),
            (
                Literal::String("hello".to_string()),
                Literal::String("world".to_string()),
                false,
            ),
            (
                Literal::Float("3.14".to_string()),
                Literal::Float("3.14".to_string()),
                true,
            ),
            (Literal::Unit, Literal::Unit, true),
        ];
        
        for (expr_lit, pat_lit, should_match) in literals {
            let expr = Expression::Literal(expr_lit);
            let pattern = Pattern::Literal(pat_lit);
            let results = match_pattern(&expr, &pattern);
            
            assert_eq!(
                !results.is_empty(),
                should_match,
                "Literal matching failed"
            );
        }
        
        println!("✓ All literal types tested successfully");
    }
}

use std::collections::{HashSet, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::pattern::{Expression, Literal, MatchArm, Pattern};

static GENSYM_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn gensym(prefix: &str) -> String {
    let counter = GENSYM_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}${}", prefix, counter)
}

#[cfg(test)]
pub fn reset_gensym() {
    GENSYM_COUNTER.store(0, Ordering::SeqCst);
}

pub fn free_vars(expr: &Expression) -> HashSet<String> {
    let mut vars = HashSet::new();
    collect_free_vars(expr, &mut vars, &HashSet::new());
    vars
}

fn collect_free_vars(
    expr: &Expression,
    free: &mut HashSet<String>,
    bound: &HashSet<String>,
) {
    match expr {
        Expression::Literal(_) => {}
        
        Expression::Var(name) => {
            if !bound.contains(name) {
                free.insert(name.clone());
            }
        }
        
        Expression::Lambda { param, body } => {
            let mut new_bound = bound.clone();
            new_bound.insert(param.clone());
            collect_free_vars(body, free, &new_bound);
        }
        
        Expression::Apply { func, arg } | Expression::LinearApply { func, arg } => {
            collect_free_vars(func, free, bound);
            collect_free_vars(arg, free, bound);
        }
        
        Expression::Let { name, value, body } => {
            collect_free_vars(value, free, bound);
            let mut new_bound = bound.clone();
            new_bound.insert(name.clone());
            collect_free_vars(body, free, &new_bound);
        }
        
        Expression::Match { expr, arms } => {
            collect_free_vars(expr, free, bound);
            for arm in arms {
                let pattern_vars = pattern_variables(&arm.pattern);
                let mut new_bound = bound.clone();
                for var in pattern_vars {
                    new_bound.insert(var);
                }
                
                if let Some(guard) = &arm.guard {
                    collect_free_vars(guard, free, &new_bound);
                }
                collect_free_vars(&arm.body, free, &new_bound);
            }
        }
        
        Expression::Tuple(exprs) | Expression::List(exprs) => {
            for e in exprs {
                collect_free_vars(e, free, bound);
            }
        }
        
        Expression::Record(fields) => {
            for (_, e) in fields {
                collect_free_vars(e, free, bound);
            }
        }
    }
}

pub fn pattern_variables(pattern: &Pattern) -> HashSet<String> {
    let mut vars = HashSet::new();
    collect_pattern_variables(pattern, &mut vars);
    vars
}

fn collect_pattern_variables(pattern: &Pattern, vars: &mut HashSet<String>) {
    match pattern {
        Pattern::Wildcard | Pattern::Literal(_) => {}
        
        Pattern::Var(name) => {
            vars.insert(name.clone());
        }
        
        Pattern::Bind { name, pattern } => {
            vars.insert(name.clone());
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

pub fn alpha_rename(
    expr: &Expression,
    old_name: &str,
    avoid_set: &HashSet<String>,
) -> (String, Expression) {
    let mut new_name = gensym(old_name);
    while avoid_set.contains(&new_name) {
        new_name = gensym(old_name);
    }
    
    let renamed = substitute_internal(expr, old_name, &Expression::Var(new_name.clone()), &HashSet::new());
    
    (new_name, renamed)
}

pub fn alpha_rename_pattern(
    pattern: &Pattern,
    old_name: &str,
    avoid_set: &HashSet<String>,
) -> (String, Pattern) {
    let mut new_name = gensym(old_name);
    while avoid_set.contains(&new_name) {
        new_name = gensym(old_name);
    }
    
    let renamed = substitute_pattern(pattern, old_name, &new_name);
    (new_name, renamed)
}

fn substitute_pattern(pattern: &Pattern, old_name: &str, new_name: &str) -> Pattern {
    match pattern {
        Pattern::Wildcard => Pattern::Wildcard,
        Pattern::Literal(lit) => Pattern::Literal(lit.clone()),
        
        Pattern::Var(name) => {
            if name == old_name {
                Pattern::Var(new_name.to_string())
            } else {
                Pattern::Var(name.clone())
            }
        }
        
        Pattern::Bind { name, pattern } => {
            let new_pattern = Box::new(substitute_pattern(pattern, old_name, new_name));
            let new_bind_name = if name == old_name {
                new_name.to_string()
            } else {
                name.clone()
            };
            Pattern::Bind {
                name: new_bind_name,
                pattern: new_pattern,
            }
        }
        
        Pattern::Tuple(patterns) => {
            Pattern::Tuple(
                patterns.iter()
                    .map(|p| substitute_pattern(p, old_name, new_name))
                    .collect()
            )
        }
        
        Pattern::List(patterns) => {
            Pattern::List(
                patterns.iter()
                    .map(|p| substitute_pattern(p, old_name, new_name))
                    .collect()
            )
        }
        
        Pattern::Constructor { name, args } => {
            Pattern::Constructor {
                name: name.clone(),
                args: args.iter()
                    .map(|p| substitute_pattern(p, old_name, new_name))
                    .collect(),
            }
        }
        
        Pattern::Record(fields) => {
            Pattern::Record(
                fields.iter()
                    .map(|(k, p)| (k.clone(), substitute_pattern(p, old_name, new_name)))
                    .collect()
            )
        }
        
        Pattern::Lambda { param_pattern, body_pattern } => {
            Pattern::Lambda {
                param_pattern: Box::new(substitute_pattern(param_pattern, old_name, new_name)),
                body_pattern: Box::new(substitute_pattern(body_pattern, old_name, new_name)),
            }
        }
        
        Pattern::Apply { func_pattern, arg_pattern } => {
            Pattern::Apply {
                func_pattern: Box::new(substitute_pattern(func_pattern, old_name, new_name)),
                arg_pattern: Box::new(substitute_pattern(arg_pattern, old_name, new_name)),
            }
        }
    }
}

pub fn substitute(
    expr: &Expression,
    var: &str,
    replacement: &Expression,
) -> Expression {
    substitute_internal(expr, var, replacement, &HashSet::new())
}

fn substitute_internal(
    expr: &Expression,
    var: &str,
    replacement: &Expression,
    bound: &HashSet<String>,
) -> Expression {
    if bound.contains(var) {
        return expr.clone();
    }
    
    match expr {
        Expression::Literal(lit) => Expression::Literal(lit.clone()),
        
        Expression::Var(name) => {
            if name == var {
                replacement.clone()
            } else {
                Expression::Var(name.clone())
            }
        }
        
        Expression::Lambda { param, body } => {
            if param == var {
                Expression::Lambda {
                    param: param.clone(),
                    body: body.clone(),
                }
            } else {
                let replacement_free_vars = free_vars(replacement);
                
                if replacement_free_vars.contains(param) {
                    let mut avoid_set = free_vars(expr);
                    avoid_set.extend(replacement_free_vars);
                    avoid_set.insert(var.to_string());
                    
                    let (new_param, renamed_body) = alpha_rename(body, param, &avoid_set);
                    
                    let new_body = substitute_internal(
                        &renamed_body,
                        var,
                        replacement,
                        &{
                            let mut new_bound = bound.clone();
                            new_bound.insert(new_param.clone());
                            new_bound
                        },
                    );
                    
                    Expression::Lambda {
                        param: new_param,
                        body: Box::new(new_body),
                    }
                } else {
                    let mut new_bound = bound.clone();
                    new_bound.insert(param.clone());
                    
                    Expression::Lambda {
                        param: param.clone(),
                        body: Box::new(substitute_internal(body, var, replacement, &new_bound)),
                    }
                }
            }
        }
        
        Expression::Apply { func, arg } => {
            Expression::Apply {
                func: Box::new(substitute_internal(func, var, replacement, bound)),
                arg: Box::new(substitute_internal(arg, var, replacement, bound)),
            }
        }
        
        Expression::LinearApply { func, arg } => {
            Expression::LinearApply {
                func: Box::new(substitute_internal(func, var, replacement, bound)),
                arg: Box::new(substitute_internal(arg, var, replacement, bound)),
            }
        }
        
        Expression::Let { name, value, body } => {
            let new_value = substitute_internal(value, var, replacement, bound);
            
            if name == var {
                Expression::Let {
                    name: name.clone(),
                    value: Box::new(new_value),
                    body: body.clone(),
                }
            } else {
                let replacement_free_vars = free_vars(replacement);
                
                if replacement_free_vars.contains(name) {
                    let mut avoid_set = free_vars(expr);
                    avoid_set.extend(replacement_free_vars);
                    avoid_set.insert(var.to_string());
                    
                    let (new_name, renamed_body) = alpha_rename(body, name, &avoid_set);
                    
                    let new_body = substitute_internal(
                        &renamed_body,
                        var,
                        replacement,
                        &{
                            let mut new_bound = bound.clone();
                            new_bound.insert(new_name.clone());
                            new_bound
                        },
                    );
                    
                    Expression::Let {
                        name: new_name,
                        value: Box::new(new_value),
                        body: Box::new(new_body),
                    }
                } else {
                    let mut new_bound = bound.clone();
                    new_bound.insert(name.clone());
                    
                    Expression::Let {
                        name: name.clone(),
                        value: Box::new(new_value),
                        body: Box::new(substitute_internal(body, var, replacement, &new_bound)),
                    }
                }
            }
        }
        
        Expression::Match { expr: match_expr, arms } => {
            let new_expr = Box::new(substitute_internal(match_expr, var, replacement, bound));
            let new_arms = arms.iter().map(|arm| {
                substitute_match_arm(arm, var, replacement, bound)
            }).collect();
            
            Expression::Match {
                expr: new_expr,
                arms: new_arms,
            }
        }
        
        Expression::Tuple(exprs) => {
            Expression::Tuple(
                exprs.iter()
                    .map(|e| substitute_internal(e, var, replacement, bound))
                    .collect()
            )
        }
        
        Expression::List(exprs) => {
            Expression::List(
                exprs.iter()
                    .map(|e| substitute_internal(e, var, replacement, bound))
                    .collect()
            )
        }
        
        Expression::Record(fields) => {
            Expression::Record(
                fields.iter()
                    .map(|(k, e)| (k.clone(), substitute_internal(e, var, replacement, bound)))
                    .collect()
            )
        }
    }
}

fn substitute_match_arm(
    arm: &MatchArm,
    var: &str,
    replacement: &Expression,
    bound: &HashSet<String>,
) -> MatchArm {
    let pattern_vars = pattern_variables(&arm.pattern);
    
    if pattern_vars.contains(var) {
        return arm.clone();
    }
    
    let replacement_free_vars = free_vars(replacement);
    let captures: Vec<_> = pattern_vars.intersection(&replacement_free_vars).cloned().collect();
    
    if captures.is_empty() {
        let mut new_bound = bound.clone();
        new_bound.extend(pattern_vars);
        
        let new_guard = arm.guard.as_ref().map(|g| {
            Box::new(substitute_internal(g, var, replacement, &new_bound))
        });
        
        let new_body = Box::new(substitute_internal(&arm.body, var, replacement, &new_bound));
        
        MatchArm {
            pattern: arm.pattern.clone(),
            guard: new_guard,
            body: new_body,
        }
    } else {
        let mut avoid_set = free_vars(&arm.body);
        avoid_set.extend(replacement_free_vars);
        avoid_set.insert(var.to_string());
        
        if let Some(guard) = &arm.guard {
            avoid_set.extend(free_vars(guard));
        }
        
        let mut renaming_map = HashMap::new();
        let mut new_pattern = arm.pattern.clone();
        
        for capture_var in captures {
            let (new_name, renamed_pattern) = alpha_rename_pattern(&new_pattern, &capture_var, &avoid_set);
            renaming_map.insert(capture_var.clone(), new_name.clone());
            new_pattern = renamed_pattern;
            avoid_set.insert(new_name);
        }
        
        let mut renamed_guard = arm.guard.clone();
        let mut renamed_body = arm.body.clone();
        
        for (old_name, new_name) in &renaming_map {
            if let Some(guard) = &renamed_guard {
                renamed_guard = Some(Box::new(substitute_internal(
                    guard,
                    old_name,
                    &Expression::Var(new_name.clone()),
                    &HashSet::new(),
                )));
            }
            renamed_body = Box::new(substitute_internal(
                &renamed_body,
                old_name,
                &Expression::Var(new_name.clone()),
                &HashSet::new(),
            ));
        }
        
        let pattern_vars = pattern_variables(&new_pattern);
        let mut new_bound = bound.clone();
        new_bound.extend(pattern_vars);
        
        let new_guard = renamed_guard.map(|g| {
            Box::new(substitute_internal(&g, var, replacement, &new_bound))
        });
        
        let new_body = Box::new(substitute_internal(&renamed_body, var, replacement, &new_bound));
        
        MatchArm {
            pattern: new_pattern,
            guard: new_guard,
            body: new_body,
        }
    }
}

pub fn substitute_many(
    expr: &Expression,
    substitutions: &HashMap<String, Expression>,
) -> Expression {
    let mut result = expr.clone();
    
    let mut sorted_vars: Vec<_> = substitutions.keys().cloned().collect();
    sorted_vars.sort();
    
    for var in sorted_vars {
        if let Some(replacement) = substitutions.get(&var) {
            result = substitute(&result, &var, replacement);
        }
    }
    
    result
}

pub fn is_well_formed(expr: &Expression) -> bool {
    is_well_formed_internal(expr, &HashSet::new())
}

fn is_well_formed_internal(expr: &Expression, bound: &HashSet<String>) -> bool {
    match expr {
        Expression::Literal(_) => true,
        
        Expression::Var(name) => bound.contains(name),
        
        Expression::Lambda { param, body } => {
            let mut new_bound = bound.clone();
            new_bound.insert(param.clone());
            is_well_formed_internal(body, &new_bound)
        }
        
        Expression::Apply { func, arg } | Expression::LinearApply { func, arg } => {
            is_well_formed_internal(func, bound) && is_well_formed_internal(arg, bound)
        }
        
        Expression::Let { name, value, body } => {
            if !is_well_formed_internal(value, bound) {
                return false;
            }
            let mut new_bound = bound.clone();
            new_bound.insert(name.clone());
            is_well_formed_internal(body, &new_bound)
        }
        
        Expression::Match { expr, arms } => {
            if !is_well_formed_internal(expr, bound) {
                return false;
            }
            
            for arm in arms {
                let pattern_vars = pattern_variables(&arm.pattern);
                let mut new_bound = bound.clone();
                new_bound.extend(pattern_vars);
                
                if let Some(guard) = &arm.guard {
                    if !is_well_formed_internal(guard, &new_bound) {
                        return false;
                    }
                }
                
                if !is_well_formed_internal(&arm.body, &new_bound) {
                    return false;
                }
            }
            
            true
        }
        
        Expression::Tuple(exprs) | Expression::List(exprs) => {
            exprs.iter().all(|e| is_well_formed_internal(e, bound))
        }
        
        Expression::Record(fields) => {
            fields.iter().all(|(_, e)| is_well_formed_internal(e, bound))
        }
    }
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

    fn let_expr(name: &str, value: Expression, body: Expression) -> Expression {
        Expression::Let {
            name: name.to_string(),
            value: Box::new(value),
            body: Box::new(body),
        }
    }

    #[test]
    fn test_gensym_unique() {
        reset_gensym();
        let name1 = gensym("x");
        let name2 = gensym("x");
        assert_ne!(name1, name2);
        assert!(name1.starts_with("x$"));
        assert!(name2.starts_with("x$"));
    }

    #[test]
    fn test_free_vars_simple() {
        let expr = var("x");
        let fv = free_vars(&expr);
        assert_eq!(fv.len(), 1);
        assert!(fv.contains("x"));
    }

    #[test]
    fn test_free_vars_lambda() {
        let expr = lambda("x", var("x"));
        let fv = free_vars(&expr);
        assert_eq!(fv.len(), 0);
    }

    #[test]
    fn test_free_vars_mixed() {
        let expr = lambda("x", apply(var("x"), var("y")));
        let fv = free_vars(&expr);
        assert_eq!(fv.len(), 1);
        assert!(fv.contains("y"));
        assert!(!fv.contains("x"));
    }

    #[test]
    fn test_substitute_var() {
        let expr = var("x");
        let result = substitute(&expr, "x", &int(42));
        assert_eq!(result, int(42));
    }

    #[test]
    fn test_substitute_no_occurrence() {
        let expr = var("x");
        let result = substitute(&expr, "y", &int(42));
        assert_eq!(result, var("x"));
    }

    #[test]
    fn test_substitute_lambda_shadowing() {
        let expr = lambda("x", var("x"));
        let result = substitute(&expr, "x", &int(42));
        assert_eq!(result, lambda("x", var("x")));
    }

    #[test]
    fn test_substitute_lambda_free() {
        let expr = lambda("x", var("y"));
        let result = substitute(&expr, "y", &int(42));
        assert_eq!(result, lambda("x", int(42)));
    }

    #[test]
    fn test_substitute_capture_avoidance() {
        reset_gensym();
        let expr = lambda("y", var("x"));
        let result = substitute(&expr, "x", &var("y"));
        
        match result {
            Expression::Lambda { param, body } => {
                assert!(param.starts_with("y$"));
                assert_eq!(*body, var("y"));
            }
            _ => panic!("Expected lambda"),
        }
    }

    #[test]
    fn test_substitute_apply() {
        let expr = apply(var("f"), var("x"));
        let result = substitute(&expr, "x", &int(42));
        assert_eq!(result, apply(var("f"), int(42)));
    }

    #[test]
    fn test_substitute_let_value() {
        let expr = let_expr("x", var("y"), var("x"));
        let result = substitute(&expr, "y", &int(42));
        
        match result {
            Expression::Let { value, .. } => {
                assert_eq!(*value, int(42));
            }
            _ => panic!("Expected let"),
        }
    }

    #[test]
    fn test_substitute_let_shadowing() {
        let expr = let_expr("x", int(1), var("x"));
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::Let { body, .. } => {
                assert_eq!(*body, var("x"));
            }
            _ => panic!("Expected let"),
        }
    }

    #[test]
    fn test_substitute_let_capture() {
        reset_gensym();
        let expr = let_expr("y", int(1), var("x"));
        let result = substitute(&expr, "x", &var("y"));
        
        match result {
            Expression::Let { name, body, .. } => {
                assert!(name.starts_with("y$"));
                assert_eq!(*body, var("y"));
            }
            _ => panic!("Expected let"),
        }
    }

    #[test]
    fn test_substitute_tuple() {
        let expr = Expression::Tuple(vec![var("x"), var("y"), var("x")]);
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::Tuple(elements) => {
                assert_eq!(elements[0], int(42));
                assert_eq!(elements[1], var("y"));
                assert_eq!(elements[2], int(42));
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_substitute_list() {
        let expr = Expression::List(vec![var("x"), int(1), var("y")]);
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::List(elements) => {
                assert_eq!(elements[0], int(42));
                assert_eq!(elements[1], int(1));
                assert_eq!(elements[2], var("y"));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_substitute_record() {
        let expr = Expression::Record(vec![
            ("field1".to_string(), var("x")),
            ("field2".to_string(), var("y")),
        ]);
        
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::Record(fields) => {
                assert_eq!(fields[0].1, int(42));
                assert_eq!(fields[1].1, var("y"));
            }
            _ => panic!("Expected record"),
        }
    }

    #[test]
    fn test_substitute_match_expr() {
        let pattern = Pattern::Var("x".to_string());
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Box::new(var("x")),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("x")),
            arms: vec![arm],
        };
        
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::Match { expr, arms } => {
                assert_eq!(*expr, int(42));
                assert_eq!(arms[0].body, Box::new(var("x")));
            }
            _ => panic!("Expected match"),
        }
    }

    #[test]
    fn test_match_pattern_shadowing() {
        let pattern = Pattern::Var("x".to_string());
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Box::new(var("x")),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("y")),
            arms: vec![arm],
        };
        
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::Match { arms, .. } => {
                assert_eq!(arms[0].body, Box::new(var("x")));
            }
            _ => panic!("Expected match"),
        }
    }

    #[test]
    fn test_match_capture_avoidance() {
        reset_gensym();
        let pattern = Pattern::Var("x".to_string());
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Box::new(var("y")),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("e")),
            arms: vec![arm],
        };
        
        let result = substitute(&expr, "y", &var("x"));
        
        match result {
            Expression::Match { arms, .. } => {
                match &arms[0].pattern {
                    Pattern::Var(name) => {
                        assert!(name.starts_with("x$"));
                        assert_eq!(arms[0].body, Box::new(var("x")));
                    }
                    _ => panic!("Expected var pattern"),
                }
            }
            _ => panic!("Expected match"),
        }
    }

    #[test]
    fn test_match_with_guard() {
        let pattern = Pattern::Var("n".to_string());
        let arm = MatchArm {
            pattern,
            guard: Some(Box::new(apply(var("gt"), apply(var("n"), int(0))))),
            body: Box::new(var("x")),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("y")),
            arms: vec![arm],
        };
        
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::Match { arms, .. } => {
                assert_eq!(arms[0].body, Box::new(int(42)));
                match &arms[0].guard {
                    Some(guard) => {
                        match guard.as_ref() {
                            Expression::Apply { func, arg: _ } => {
                                match func.as_ref() {
                                    Expression::Apply { arg: n_arg, .. } => {
                                        assert_eq!(**n_arg, var("n"));
                                    }
                                    _ => {}
                                }
                            }
                            _ => panic!("Expected apply in guard"),
                        }
                    }
                    None => panic!("Expected guard"),
                }
            }
            _ => panic!("Expected match"),
        }
    }

    #[test]
    fn test_is_well_formed_closed() {
        let expr = lambda("x", var("x"));
        assert!(is_well_formed(&expr));
    }

    #[test]
    fn test_is_well_formed_open() {
        let expr = var("x");
        assert!(!is_well_formed(&expr));
    }

    #[test]
    fn test_is_well_formed_let() {
        let expr = let_expr("x", int(42), var("x"));
        assert!(is_well_formed(&expr));
    }

    #[test]
    fn test_is_well_formed_nested() {
        let expr = lambda("x", apply(var("x"), var("y")));
        assert!(!is_well_formed(&expr));
    }

    #[test]
    fn test_substitute_preserves_well_formed() {
        let expr = lambda("x", var("x"));
        assert!(is_well_formed(&expr));
        
        let result = substitute(&expr, "y", &int(42));
        assert!(is_well_formed(&result));
    }

    #[test]
    fn test_substitute_closes_open_term() {
        let expr = var("x");
        assert!(!is_well_formed(&expr));
        
        let result = substitute(&expr, "x", &int(42));
        assert!(is_well_formed(&result));
    }

    #[test]
    fn test_substitute_many() {
        let expr = apply(apply(var("f"), var("x")), var("y"));
        
        let mut subs = HashMap::new();
        subs.insert("f".to_string(), var("g"));
        subs.insert("x".to_string(), int(1));
        subs.insert("y".to_string(), int(2));
        
        let result = substitute_many(&expr, &subs);
        
        assert_eq!(result, apply(apply(var("g"), int(1)), int(2)));
    }

    #[test]
    fn test_alpha_rename_simple() {
        reset_gensym();
        let expr = var("x");
        let avoid = HashSet::new();
        
        let (new_name, renamed) = alpha_rename(&expr, "x", &avoid);
        
        assert!(new_name.starts_with("x$"));
        assert_eq!(renamed, var(&new_name));
    }

    #[test]
    fn test_alpha_rename_avoids_conflicts() {
        reset_gensym();
        let expr = var("x");
        
        let mut avoid = HashSet::new();
        avoid.insert("x$0".to_string());
        avoid.insert("x$1".to_string());
        
        let (new_name, _) = alpha_rename(&expr, "x", &avoid);
        
        assert!(!avoid.contains(&new_name));
    }

    #[test]
    fn test_pattern_variables_simple() {
        let pattern = Pattern::Var("x".to_string());
        let vars = pattern_variables(&pattern);
        
        assert_eq!(vars.len(), 1);
        assert!(vars.contains("x"));
    }

    #[test]
    fn test_pattern_variables_tuple() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Var("x".to_string()),
            Pattern::Var("y".to_string()),
            Pattern::Wildcard,
        ]);
        
        let vars = pattern_variables(&pattern);
        
        assert_eq!(vars.len(), 2);
        assert!(vars.contains("x"));
        assert!(vars.contains("y"));
    }

    #[test]
    fn test_pattern_variables_bind() {
        let pattern = Pattern::Bind {
            name: "whole".to_string(),
            pattern: Box::new(Pattern::Tuple(vec![
                Pattern::Var("x".to_string()),
                Pattern::Var("y".to_string()),
            ])),
        };
        
        let vars = pattern_variables(&pattern);
        
        assert_eq!(vars.len(), 3);
        assert!(vars.contains("whole"));
        assert!(vars.contains("x"));
        assert!(vars.contains("y"));
    }

    #[test]
    fn test_linear_application_substitution() {
        let expr = Expression::LinearApply {
            func: Box::new(var("f")),
            arg: Box::new(var("x")),
        };
        
        let result = substitute(&expr, "x", &int(42));
        
        match result {
            Expression::LinearApply { func, arg } => {
                assert_eq!(*func, var("f"));
                assert_eq!(*arg, int(42));
            }
            _ => panic!("Expected linear apply"),
        }
    }

    #[test]
    fn test_substitution_determinism() {
        reset_gensym();
        let expr = lambda("y", var("x"));
        
        let result1 = substitute(&expr, "x", &var("y"));
        
        reset_gensym();
        let result2 = substitute(&expr, "x", &var("y"));
        
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_church_numeral_substitution() {
        let two = lambda("f", lambda("x", 
            apply(var("f"), apply(var("f"), var("x")))
        ));
        
        let result = substitute(&two, "add", &var("plus"));
        
        assert!(is_well_formed(&result));
    }

    #[test]
    fn test_nested_let_substitution() {
        let expr = let_expr("x", int(1),
            let_expr("y", var("x"),
                apply(apply(var("+"), var("x")), var("y"))
            )
        );
        
        let result = substitute(&expr, "+", &var("add"));
        
        match result {
            Expression::Let { body, .. } => {
                match *body {
                    Expression::Let { body: inner_body, .. } => {
                        match *inner_body {
                            Expression::Apply { func, .. } => {
                                match *func {
                                    Expression::Apply { func: op, .. } => {
                                        assert_eq!(*op, var("add"));
                                    }
                                    _ => panic!("Expected nested apply"),
                                }
                            }
                            _ => panic!("Expected apply"),
                        }
                    }
                    _ => panic!("Expected inner let"),
                }
            }
            _ => panic!("Expected let"),
        }
    }

    #[test]
    fn test_capture_in_nested_lambda() {
        reset_gensym();
        let expr = lambda("x", lambda("y", lambda("z", var("w"))));
        let replacement = lambda("a", var("y"));
        
        let result = substitute(&expr, "w", &replacement);
        
        match result {
            Expression::Lambda { param, body } => {
                assert_eq!(param, "x");
                match *body {
                    Expression::Lambda { param: param_y, body: inner_body } => {
                        assert!(param_y.starts_with("y$"));
                        match *inner_body {
                            Expression::Lambda { body: innermost, .. } => {
                                assert_eq!(*innermost, lambda("a", var("y")));
                            }
                            _ => panic!("Expected innermost lambda"),
                        }
                    }
                    _ => panic!("Expected middle lambda"),
                }
            }
            _ => panic!("Expected outer lambda"),
        }
    }

    #[test]
    fn test_substitution_idempotence() {
        let expr = lambda("x", var("y"));
        
        let result1 = substitute(&expr, "y", &int(42));
        let result2 = substitute(&result1, "y", &int(42));
        
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_no_spurious_renaming() {
        reset_gensym();
        let expr = lambda("x", var("x"));
        let result = substitute(&expr, "y", &int(42));
        
        assert_eq!(result, lambda("x", var("x")));
    }

    #[test]
    fn test_multiple_occurrences() {
        let expr = apply(
            apply(var("x"), var("x")),
            apply(var("x"), var("y"))
        );
        
        let result = substitute(&expr, "x", &int(42));
        
        assert_eq!(
            result,
            apply(
                apply(int(42), int(42)),
                apply(int(42), var("y"))
            )
        );
    }

    #[test]
    fn test_constructor_pattern_substitution() {
        let pattern = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![Pattern::Var("x".to_string())],
        };
        
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Box::new(var("x")),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("opt")),
            arms: vec![arm],
        };
        
        let result = substitute(&expr, "opt", &var("value"));
        
        match result {
            Expression::Match { expr, arms } => {
                assert_eq!(*expr, var("value"));
                assert_eq!(arms[0].body, Box::new(var("x")));
            }
            _ => panic!("Expected match"),
        }
    }

    #[test]
    fn test_record_pattern_substitution() {
        let pattern = Pattern::Record(vec![
            ("x".to_string(), Pattern::Var("a".to_string())),
            ("y".to_string(), Pattern::Var("b".to_string())),
        ]);
        
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Box::new(apply(var("a"), var("b"))),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("record")),
            arms: vec![arm],
        };
        
        let result = substitute(&expr, "record", &var("r"));
        
        match result {
            Expression::Match { expr, arms } => {
                assert_eq!(*expr, var("r"));
                assert_eq!(arms[0].body, Box::new(apply(var("a"), var("b"))));
            }
            _ => panic!("Expected match"),
        }
    }

    #[test]
    fn test_deeply_nested_structure() {
        let expr = Expression::Tuple(vec![
            Expression::List(vec![
                lambda("x", var("y")),
                lambda("z", var("y")),
            ]),
            var("y"),
        ]);
        
        let result = substitute(&expr, "y", &int(42));
        
        match result {
            Expression::Tuple(elements) => {
                assert_eq!(elements[1], int(42));
                match &elements[0] {
                    Expression::List(lambdas) => {
                        assert_eq!(lambdas.len(), 2);
                        match &lambdas[0] {
                            Expression::Lambda { body, .. } => {
                                assert_eq!(**body, int(42));
                            }
                            _ => panic!("Expected lambda"),
                        }
                    }
                    _ => panic!("Expected list"),
                }
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_free_vars_in_match() {
        let pattern = Pattern::Var("x".to_string());
        let arm = MatchArm {
            pattern,
            guard: Some(Box::new(var("p"))),
            body: Box::new(apply(var("x"), var("y"))),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("z")),
            arms: vec![arm],
        };
        
        let fv = free_vars(&expr);
        
        assert!(fv.contains("y"));
        assert!(fv.contains("z"));
        assert!(fv.contains("p"));
        assert!(!fv.contains("x"));
    }

    #[test]
    fn test_comprehensive_substitution() {
        println!("\n=== Comprehensive Substitution Tests ===");
        
        reset_gensym();
        
        let test1 = var("x");
        let result1 = substitute(&test1, "x", &int(42));
        assert_eq!(result1, int(42));
        println!("✓ Test 1: Simple variable replacement");
        
        let test2 = lambda("x", var("x"));
        let result2 = substitute(&test2, "x", &int(42));
        assert_eq!(result2, lambda("x", var("x")));
        println!("✓ Test 2: Lambda shadowing");
        
        reset_gensym();
        let test3 = lambda("y", var("x"));
        let result3 = substitute(&test3, "x", &int(42));
        assert!(is_well_formed(&result3));
        println!("✓ Test 3: Substitution in lambda");
        
        let test4 = let_expr("x", var("y"), var("x"));
        let result4 = substitute(&test4, "y", &int(42));
        match result4 {
            Expression::Let { value, .. } => {
                assert_eq!(*value, int(42));
            }
            _ => panic!("Expected let"),
        }
        println!("✓ Test 4: Let binding substitution");
        
        let test5 = lambda("f", 
            lambda("x", 
                apply(var("f"), apply(var("f"), var("x")))
            )
        );
        let result5 = substitute(&test5, "g", &var("h"));
        assert!(is_well_formed(&result5));
        println!("✓ Test 5: Complex nested lambda");
        
        let test6 = apply(apply(var("a"), var("b")), var("c"));
        let mut subs = HashMap::new();
        subs.insert("a".to_string(), int(1));
        subs.insert("b".to_string(), int(2));
        subs.insert("c".to_string(), int(3));
        let result6 = substitute_many(&test6, &subs);
        assert_eq!(result6, apply(apply(int(1), int(2)), int(3)));
        println!("✓ Test 6: Multiple simultaneous substitutions");
        
        reset_gensym();
        let pattern = Pattern::Var("x".to_string());
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Box::new(var("y")),
        };
        let test7 = Expression::Match {
            expr: Box::new(lambda("e", var("e"))),
            arms: vec![arm],
        };
        let result7 = substitute(&test7, "y", &int(42));
        assert!(is_well_formed(&result7));
        println!("✓ Test 7: Match expression substitution");
        
        let test8 = lambda("x", lambda("y", apply(var("x"), var("y"))));
        assert!(is_well_formed(&test8));
        let result8 = substitute(&test8, "z", &int(42));
        assert!(is_well_formed(&result8));
        println!("✓ Test 8: Well-formedness preservation");
        
        let test9 = Expression::Tuple(vec![
            var("x"),
            Expression::List(vec![var("x"), var("y")]),
        ]);
        let result9 = substitute(&test9, "x", &int(42));
        match result9 {
            Expression::Tuple(elements) => {
                assert_eq!(elements[0], int(42));
                match &elements[1] {
                    Expression::List(items) => {
                        assert_eq!(items[0], int(42));
                        assert_eq!(items[1], var("y"));
                    }
                    _ => panic!("Expected list"),
                }
            }
            _ => panic!("Expected tuple"),
        }
        println!("✓ Test 9: Tuple and list substitution");
        
        let test10 = Expression::Record(vec![
            ("field1".to_string(), var("x")),
            ("field2".to_string(), var("y")),
        ]);
        let result10 = substitute(&test10, "x", &int(42));
        match result10 {
            Expression::Record(fields) => {
                assert_eq!(fields[0].1, int(42));
                assert_eq!(fields[1].1, var("y"));
            }
            _ => panic!("Expected record"),
        }
        println!("✓ Test 10: Record substitution");
        
        println!("\n=== All comprehensive tests passed ===");
    }

    #[test]
    fn test_substitution_commutativity() {
        let expr = apply(var("x"), var("y"));
        
        let result1 = substitute(&substitute(&expr, "x", &int(1)), "y", &int(2));
        let result2 = substitute(&substitute(&expr, "y", &int(2)), "x", &int(1));
        
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_match_guard_capture_prevention() {
        reset_gensym();
        let pattern = Pattern::Var("x".to_string());
        let arm = MatchArm {
            pattern,
            guard: Some(Box::new(var("x$0"))),
            body: Box::new(var("y")),
        };
        
        let expr = Expression::Match {
            expr: Box::new(var("e")),
            arms: vec![arm],
        };
        
        let result = substitute(&expr, "y", &var("x"));
        
        match result {
            Expression::Match { arms, .. } => {
                match &arms[0].pattern {
                    Pattern::Var(name) => {
                        assert!(name.starts_with("x$"));
                        assert_ne!(name, "x$0");
                    }
                    _ => panic!("Expected var pattern"),
                }
                
                match &arms[0].guard {
                    Some(guard) => {
                        assert_eq!(**guard, var("x$0"));
                    }
                    None => panic!("Expected guard"),
                }
                
                assert_eq!(arms[0].body, Box::new(var("x")));
            }
            _ => panic!("Expected match"),
        }
    }

    #[test]
    fn test_alpha_equivalence_preservation() {
        reset_gensym();
        let expr1 = lambda("x", apply(var("x"), var("z")));
        let expr2 = lambda("y", apply(var("y"), var("z")));
        
        let result1 = substitute(&expr1, "z", &int(42));
        let result2 = substitute(&expr2, "z", &int(42));
        
        match (result1, result2) {
            (
                Expression::Lambda { body: body1, .. },
                Expression::Lambda { body: body2, .. }
            ) => {
                match (*body1, *body2) {
                    (
                        Expression::Apply { arg: arg1, .. },
                        Expression::Apply { arg: arg2, .. }
                    ) => {
                        assert_eq!(arg1, arg2);
                        assert_eq!(*arg1, int(42));
                    }
                    _ => panic!("Expected applications"),
                }
            }
            _ => panic!("Expected lambdas"),
        }
    }
}

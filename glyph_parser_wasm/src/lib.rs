//! GΛLYPH Parser WebAssembly Module
//!
//! This module provides WebAssembly bindings for the GΛLYPH parser,
//! allowing browser-based game generation with functional programming expressions.

use wasm_bindgen::prelude::*;
use serde_wasm_bindgen;
use glyph_parser::parse as parse_glyph;
use glyph_parser::canonical_serialize;
use glyph_parser::Expression;

// Import console.log for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Initialize console error panic hook
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// Public API
// ============================================================================

/// Parse a GΛLYPH expression string into an AST
#[wasm_bindgen]
pub fn parse_expression(input: &str) -> Result<JsValue, JsValue> {
    log(&format!("Parsing expression: {}", input));

    match parse_glyph(input) {
        Ok(expr) => {
            log("Successfully parsed expression");
            serde_wasm_bindgen::to_value(&WasmExpression::from(expr))
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
        Err(e) => {
            log(&format!("Parse error: {}", e));
            Err(JsValue::from_str(&format!("Parse error: {}", e)))
        }
    }
}

/// Validate a GΛLYPH expression
#[wasm_bindgen]
pub fn validate_expression(input: &str) -> JsValue {
    log(&format!("Validating expression: {}", input));

    match parse_glyph(input) {
        Ok(expr) => {
            // Additional validation logic
            let validation_result = validate_glyph_structure(&expr);
            serde_wasm_bindgen::to_value(&validation_result)
                .unwrap_or_else(|_| JsValue::from_str("Serialization error"))
        }
        Err(e) => {
            let error_result = ValidationResult {
                is_valid: false,
                errors: vec![format!("Parse error: {}", e)],
                warnings: vec![],
            };
            serde_wasm_bindgen::to_value(&error_result)
                .unwrap_or_else(|_| JsValue::from_str("Serialization error"))
        }
    }
}

/// Canonical serialize an expression to bytes
#[wasm_bindgen]
pub fn serialize_expression(expr_js: JsValue) -> Result<Vec<u8>, JsValue> {
    let wasm_expr: WasmExpression = serde_wasm_bindgen::from_value(expr_js)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

    let expr: Expression = wasm_expr.into();

    canonical_serialize(&expr)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get the content hash of an expression
#[wasm_bindgen]
pub fn get_expression_hash(expr_js: JsValue) -> Result<String, JsValue> {
    let wasm_expr: WasmExpression = serde_wasm_bindgen::from_value(expr_js)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

    let expr: Expression = wasm_expr.into();
    let serialized = canonical_serialize(&expr)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

    // Compute SHA-256 hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    let result = hasher.finalize();

    Ok(hex::encode(result))
}

/// Check if an expression is a game-specific expression
#[wasm_bindgen]
pub fn is_game_expression(expr_js: JsValue) -> bool {
    if let Ok(wasm_expr) = serde_wasm_bindgen::from_value::<WasmExpression>(expr_js) {
        is_game_specific_expression(&wasm_expr)
    } else {
        false
    }
}

/// Extract game components from an expression
#[wasm_bindgen]
pub fn extract_game_components(expr_js: JsValue) -> Result<JsValue, JsValue> {
    let wasm_expr: WasmExpression = serde_wasm_bindgen::from_value(expr_js)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

    let components = extract_components_from_expression(&wasm_expr);

    serde_wasm_bindgen::to_value(&components)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// ============================================================================
// WebAssembly-compatible Data Structures
// ============================================================================

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WasmExpression {
    Literal {
        value: WasmLiteral,
    },
    Variable {
        name: String,
    },
    Lambda {
        parameter: String,
        body: Box<WasmExpression>,
    },
    Application {
        function: Box<WasmExpression>,
        argument: Box<WasmExpression>,
    },
    LinearApplication {
        function: Box<WasmExpression>,
        argument: Box<WasmExpression>,
    },
    Let {
        name: String,
        value: Box<WasmExpression>,
        body: Box<WasmExpression>,
    },
    Match {
        expression: Box<WasmExpression>,
        arms: Vec<WasmMatchArm>,
    },
    Tuple {
        elements: Vec<WasmExpression>,
    },
    List {
        elements: Vec<WasmExpression>,
    },
    Record {
        fields: std::collections::HashMap<String, WasmExpression>,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WasmLiteral {
    Integer { value: i64 },
    Float { value: String },
    String { value: String },
    Boolean { value: bool },
    Unit,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct WasmMatchArm {
    pub pattern: WasmPattern,
    pub guard: Option<Box<WasmExpression>>,
    pub body: Box<WasmExpression>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WasmPattern {
    Wildcard,
    Variable { name: String },
    Literal { literal: WasmLiteral },
    Tuple { patterns: Vec<WasmPattern> },
    Constructor { name: String, arguments: Vec<WasmPattern> },
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameComponents {
    pub narrative: Option<String>,
    pub mechanics: Option<String>,
    pub assets: Option<String>,
    pub balance: Option<String>,
    pub is_game_lambda: bool,
}

// ============================================================================
// Conversion Functions
// ============================================================================

impl From<Expression> for WasmExpression {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Literal(lit) => WasmExpression::Literal {
                value: WasmLiteral::from(lit),
            },
            Expression::Var(name) => WasmExpression::Variable { name },
            Expression::Lambda { param, body } => WasmExpression::Lambda {
                parameter: param,
                body: Box::new((*body).into()),
            },
            Expression::Apply { func, arg } => WasmExpression::Application {
                function: Box::new((*func).into()),
                argument: Box::new((*arg).into()),
            },
            Expression::LinearApply { func, arg } => WasmExpression::LinearApplication {
                function: Box::new((*func).into()),
                argument: Box::new((*arg).into()),
            },
            Expression::Let { name, value, body } => WasmExpression::Let {
                name,
                value: Box::new((*value).into()),
                body: Box::new((*body).into()),
            },
            Expression::Match { expr, arms } => WasmExpression::Match {
                expression: Box::new((*expr).into()),
                arms: arms.into_iter().map(WasmMatchArm::from).collect(),
            },
            Expression::Tuple(elements) => WasmExpression::Tuple {
                elements: elements.into_iter().map(WasmExpression::from).collect(),
            },
            Expression::List(elements) => WasmExpression::List {
                elements: elements.into_iter().map(WasmExpression::from).collect(),
            },
            Expression::Record(fields) => WasmExpression::Record {
                fields: fields
                    .into_iter()
                    .map(|(k, v)| (k, WasmExpression::from(v)))
                    .collect(),
            },
        }
    }
}

impl From<glyph_parser::Literal> for WasmLiteral {
    fn from(lit: glyph_parser::Literal) -> Self {
        match lit {
            glyph_parser::Literal::Int(i) => WasmLiteral::Integer { value: i },
            glyph_parser::Literal::Float(f) => WasmLiteral::Float { value: f },
            glyph_parser::Literal::String(s) => WasmLiteral::String { value: s },
            glyph_parser::Literal::Bool(b) => WasmLiteral::Boolean { value: b },
            glyph_parser::Literal::Unit => WasmLiteral::Unit,
        }
    }
}

impl From<glyph_parser::MatchArm> for WasmMatchArm {
    fn from(arm: glyph_parser::MatchArm) -> Self {
        WasmMatchArm {
            pattern: WasmPattern::from(arm.pattern),
            guard: arm.guard.map(|g| Box::new(WasmExpression::from(*g))),
            body: Box::new(WasmExpression::from(*arm.body)),
        }
    }
}

impl From<glyph_parser::Pattern> for WasmPattern {
    fn from(pattern: glyph_parser::Pattern) -> Self {
        match pattern {
            glyph_parser::Pattern::Wildcard => WasmPattern::Wildcard,
            glyph_parser::Pattern::Var(name) => WasmPattern::Variable { name },
            glyph_parser::Pattern::Literal(lit) => WasmPattern::Literal {
                literal: WasmLiteral::from(lit),
            },
            glyph_parser::Pattern::Tuple(patterns) => WasmPattern::Tuple {
                patterns: patterns.into_iter().map(WasmPattern::from).collect(),
            },
            glyph_parser::Pattern::Constructor { name, args } => WasmPattern::Constructor {
                name,
                arguments: args.into_iter().map(WasmPattern::from).collect(),
            },
        }
    }
}

impl From<WasmExpression> for Expression {
    fn from(wasm_expr: WasmExpression) -> Self {
        match wasm_expr {
            WasmExpression::Literal { value } => Expression::Literal(value.into()),
            WasmExpression::Variable { name } => Expression::Var(name),
            WasmExpression::Lambda { parameter, body } => Expression::Lambda {
                param: parameter,
                body: Box::new((*body).into()),
            },
            WasmExpression::Application { function, argument } => Expression::Apply {
                func: Box::new((*function).into()),
                arg: Box::new((*argument).into()),
            },
            WasmExpression::LinearApplication { function, argument } => Expression::LinearApply {
                func: Box::new((*function).into()),
                arg: Box::new((*argument).into()),
            },
            WasmExpression::Let { name, value, body } => Expression::Let {
                name,
                value: Box::new((*value).into()),
                body: Box::new((*body).into()),
            },
            WasmExpression::Match { expression, arms } => Expression::Match {
                expr: Box::new((*expression).into()),
                arms: arms.into_iter().map(glyph_parser::MatchArm::from).collect(),
            },
            WasmExpression::Tuple { elements } => Expression::Tuple(
                elements.into_iter().map(Expression::from).collect(),
            ),
            WasmExpression::List { elements } => Expression::List(
                elements.into_iter().map(Expression::from).collect(),
            ),
            WasmExpression::Record { fields } => Expression::Record(
                fields
                    .into_iter()
                    .map(|(k, v)| (k, Expression::from(v)))
                    .collect(),
            ),
        }
    }
}

impl From<WasmLiteral> for glyph_parser::Literal {
    fn from(lit: WasmLiteral) -> Self {
        match lit {
            WasmLiteral::Integer { value } => glyph_parser::Literal::Int(value),
            WasmLiteral::Float { value } => glyph_parser::Literal::Float(value),
            WasmLiteral::String { value } => glyph_parser::Literal::String(value),
            WasmLiteral::Boolean { value } => glyph_parser::Literal::Bool(value),
            WasmLiteral::Unit => glyph_parser::Literal::Unit,
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn validate_glyph_structure(expr: &Expression) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Basic structure validation
    validate_expression_structure(expr, &mut errors, &mut warnings);

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}

fn validate_expression_structure(
    expr: &Expression,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    match expr {
        Expression::Lambda { param, body } => {
            if param.is_empty() {
                errors.push("Lambda parameter cannot be empty".to_string());
            }

            // Validate body recursively
            validate_expression_structure(body, errors, warnings);
        }
        Expression::Apply { func, arg } => {
            validate_expression_structure(func, errors, warnings);
            validate_expression_structure(arg, errors, warnings);
        }
        Expression::LinearApply { func, arg } => {
            validate_expression_structure(func, errors, warnings);
            validate_expression_structure(arg, errors, warnings);
        }
        Expression::Let { name, value, body } => {
            if name.is_empty() {
                errors.push("Let binding name cannot be empty".to_string());
            }
            validate_expression_structure(value, errors, warnings);
            validate_expression_structure(body, errors, warnings);
        }
        Expression::Match { expr: match_expr, arms } => {
            validate_expression_structure(match_expr, errors, warnings);
            for arm in arms {
                validate_expression_structure(&arm.body, errors, warnings);
                if let Some(guard) = &arm.guard {
                    validate_expression_structure(guard, errors, warnings);
                }
            }
        }
        Expression::Tuple(elements) => {
            for element in elements {
                validate_expression_structure(element, errors, warnings);
            }
        }
        Expression::List(elements) => {
            for element in elements {
                validate_expression_structure(element, errors, warnings);
            }
        }
        Expression::Record(fields) => {
            for (_, value) in fields {
                validate_expression_structure(value, errors, warnings);
            }
        }
        _ => {} // Literals and variables are always valid
    }
}

fn is_game_specific_expression(expr: &WasmExpression) -> bool {
    // Check if this is a game-specific lambda expression
    match expr {
        WasmExpression::Lambda { parameter, body } => {
            parameter == "game" ||
            parameter == "narrative" ||
            parameter == "mechanics" ||
            parameter == "assets" ||
            parameter == "balance"
        }
        _ => false,
    }
}

fn extract_components_from_expression(expr: &WasmExpression) -> GameComponents {
    let mut components = GameComponents {
        narrative: None,
        mechanics: None,
        assets: None,
        balance: None,
        is_game_lambda: false,
    };

    // Extract components from nested expressions
    extract_components_recursive(expr, &mut components);

    components
}

fn extract_components_recursive(expr: &WasmExpression, components: &mut GameComponents) {
    match expr {
        WasmExpression::Lambda { parameter, body } => {
            if parameter == "game" {
                components.is_game_lambda = true;
            } else if parameter == "narrative" {
                components.narrative = Some(serialize_component(&body));
            } else if parameter == "mechanics" {
                components.mechanics = Some(serialize_component(&body));
            } else if parameter == "assets" {
                components.assets = Some(serialize_component(&body));
            } else if parameter == "balance" {
                components.balance = Some(serialize_component(&body));
            }

            extract_components_recursive(body, components);
        }
        WasmExpression::Application { function, argument } => {
            extract_components_recursive(function, components);
            extract_components_recursive(argument, components);
        }
        WasmExpression::LinearApplication { function, argument } => {
            extract_components_recursive(function, components);
            extract_components_recursive(argument, components);
        }
        WasmExpression::Let { value, body, .. } => {
            extract_components_recursive(value, components);
            extract_components_recursive(body, components);
        }
        WasmExpression::Match { expression, arms } => {
            extract_components_recursive(expression, components);
            for arm in arms {
                extract_components_recursive(&arm.body, components);
                if let Some(guard) = &arm.guard {
                    extract_components_recursive(guard, components);
                }
            }
        }
        WasmExpression::Tuple { elements } => {
            for element in elements {
                extract_components_recursive(element, components);
            }
        }
        WasmExpression::List { elements } => {
            for element in elements {
                extract_components_recursive(element, components);
            }
        }
        WasmExpression::Record { fields } => {
            for (_, value) in fields {
                extract_components_recursive(value, components);
            }
        }
        _ => {} // Literals and variables don't contain components
    }
}

fn serialize_component(expr: &WasmExpression) -> String {
    serde_json::to_string(expr).unwrap_or_else(|_| "Serialization error".to_string())
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Check if the expression contains circular references
#[wasm_bindgen]
pub fn check_circular_references(expr_js: JsValue) -> JsValue {
    // This would implement cycle detection
    // For now, return a simple result
    let result = serde_json::json!({
        "has_cycles": false,
        "cycle_path": []
    });
    serde_wasm_bindgen::to_value(&result).unwrap_or_else(|_| JsValue::from_str("Error"))
}

/// Simplify an expression by applying basic reduction rules
#[wasm_bindgen]
pub fn simplify_expression(expr_js: JsValue) -> Result<JsValue, JsValue> {
    // This would implement expression simplification
    // For now, just return the original expression
    Ok(expr_js)
}

/// Get the size of an expression (number of nodes)
#[wasm_bindgen]
pub fn get_expression_size(expr_js: JsValue) -> Result<u32, JsValue> {
    let wasm_expr: WasmExpression = serde_wasm_bindgen::from_value(expr_js)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

    let size = count_expression_nodes(&wasm_expr);
    Ok(size)
}

fn count_expression_nodes(expr: &WasmExpression) -> u32 {
    let mut count = 1; // Count this node

    match expr {
        WasmExpression::Lambda { body, .. } => {
            count += count_expression_nodes(body);
        }
        WasmExpression::Application { function, argument } => {
            count += count_expression_nodes(function);
            count += count_expression_nodes(argument);
        }
        WasmExpression::LinearApplication { function, argument } => {
            count += count_expression_nodes(function);
            count += count_expression_nodes(argument);
        }
        WasmExpression::Let { value, body, .. } => {
            count += count_expression_nodes(value);
            count += count_expression_nodes(body);
        }
        WasmExpression::Match { expression, arms } => {
            count += count_expression_nodes(expression);
            for arm in arms {
                count += count_expression_nodes(&arm.body);
                if let Some(guard) = &arm.guard {
                    count += count_expression_nodes(guard);
                }
            }
        }
        WasmExpression::Tuple { elements } => {
            for element in elements {
                count += count_expression_nodes(element);
            }
        }
        WasmExpression::List { elements } => {
            for element in elements {
                count += count_expression_nodes(element);
            }
        }
        WasmExpression::Record { fields } => {
            for (_, value) in fields {
                count += count_expression_nodes(value);
            }
        }
        _ => {} // Literals and variables have no children
    }

    count
}
//! Game-specific extensions for GΛLYPH parser
//!
//! This module adds game-specific expression types and parsing capabilities
//! to support the multi-LLM game generation system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Expression, Literal, Token, LexError, ParseError, Lexer, Parser};

// ============================================================================
// Game-Specific Expression Types
// ============================================================================

/// Game-specific expression extensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameExpression {
    /// Core game lambda: λgame.<expression>
    GameLambda {
        body: Box<Expression>,
    },

    /// Game component: λnarrative.<expression>, λmechanics.<expression>, etc.
    ComponentLambda {
        component: GameComponent,
        body: Box<Expression>,
    },

    /// Game rule definition
    GameRule {
        name: String,
        condition: Option<Box<Expression>>,
        effect: Box<Expression>,
    },

    /// Game state transition
    GameTransition {
        from_state: Option<String>,
        to_state: String,
        action: Option<Box<Expression>>,
    },

    /// Game balance expression
    BalanceExpr {
        score: f64,
        factors: Vec<BalanceFactor>,
    },

    /// Asset specification
    AssetSpec {
        asset_type: AssetType,
        properties: HashMap<String, Expression>,
    },

    /// Game narrative structure
    NarrativeStructure {
        acts: Vec<NarrativeAct>,
        choices: Vec<GameChoice>,
    },

    /// Game mechanics specification
    MechanicsSpec {
        turn_based: Option<bool>,
        player_count: Option<i64>,
        win_condition: Option<Box<Expression>>,
        lose_condition: Option<Box<Expression>>,
    },
}

/// Game component types for lambda expressions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameComponent {
    Narrative,
    Mechanics,
    Assets,
    Balance,
}

/// Balance factors for game balancing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalanceFactor {
    pub name: String,
    pub weight: f64,
    pub expression: Expression,
}

/// Asset types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Sprite,
    Sound,
    Music,
    Texture,
    Model,
    Animation,
    UI,
}

/// Narrative act structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NarrativeAct {
    pub name: String,
    pub description: Option<String>,
    pub scenes: Vec<String>,
}

/// Game choice/branching narrative
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameChoice {
    pub id: String,
    pub text: String,
    pub consequence: Expression,
    pub requirements: Option<Vec<Expression>>,
}

/// Extended token set for game expressions
#[derive(Debug, Clone, PartialEq)]
pub enum GameToken {
    // Core game tokens
    Game,
    Narrative,
    Mechanics,
    Assets,
    Balance,

    // Game structure tokens
    Rule,
    Transition,
    Asset,
    State,

    // Asset type tokens
    Sprite,
    Sound,
    Music,
    Texture,
    Model,
    Animation,
    UI,

    // Game mechanics tokens
    TurnBased,
    Players,
    Win,
    Lose,
    Score,

    // Narrative tokens
    Act,
    Scene,
    Choice,
    Branch,
}

// ============================================================================
// Extended Lexer
// ============================================================================

pub struct GameLexer {
    base: Lexer,
}

impl GameLexer {
    pub fn new(input: &str) -> Self {
        Self {
            base: Lexer::new(input),
        }
    }

    /// Check if an identifier is a game keyword
    fn is_game_keyword(&self, ident: &str) -> Option<Token> {
        match ident {
            "game" => Some(Token::Ident("game".to_string())),
            "narrative" => Some(Token::Ident("narrative".to_string())),
            "mechanics" => Some(Token::Ident("mechanics".to_string())),
            "assets" => Some(Token::Ident("assets".to_string())),
            "balance" => Some(Token::Ident("balance".to_string())),
            "rule" => Some(Token::Ident("rule".to_string())),
            "transition" => Some(Token::Ident("transition".to_string())),
            "asset" => Some(Token::Ident("asset".to_string())),
            "state" => Some(Token::Ident("state".to_string())),
            "sprite" => Some(Token::Ident("sprite".to_string())),
            "sound" => Some(Token::Ident("sound".to_string())),
            "music" => Some(Token::Ident("music".to_string())),
            "texture" => Some(Token::Ident("texture".to_string())),
            "model" => Some(Token::Ident("model".to_string())),
            "animation" => Some(Token::Ident("animation".to_string())),
            "ui" => Some(Token::Ident("ui".to_string())),
            "turn_based" => Some(Token::Ident("turn_based".to_string())),
            "players" => Some(Token::Ident("players".to_string())),
            "win" => Some(Token::Ident("win".to_string())),
            "lose" => Some(Token::Ident("lose".to_string())),
            "score" => Some(Token::Ident("score".to_string())),
            "act" => Some(Token::Ident("act".to_string())),
            "scene" => Some(Token::Ident("scene".to_string())),
            "choice" => Some(Token::Ident("choice".to_string())),
            "branch" => Some(Token::Ident("branch".to_string())),
            _ => None,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // Try base lexer first
        match self.base.next_token() {
            Ok(Token::Ident(ident)) => {
                // Check if it's a game keyword
                if let Some(game_token) = self.is_game_keyword(&ident) {
                    Ok(game_token)
                } else {
                    Ok(Token::Ident(ident))
                }
            }
            result => result,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }
}

// ============================================================================
// Extended Parser
// ============================================================================

pub struct GameParser {
    base: Parser,
}

impl GameParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            base: Parser::new(tokens),
        }
    }

    /// Parse game-specific lambda expressions
    pub fn parse_game_lambda(&mut self) -> Result<Expression, ParseError> {
        // Expect λgame.<body>
        if self.is_current_game_component() {
            self.parse_component_lambda()
        } else if self.is_current_game_token() {
            self.parse_core_game_lambda()
        } else {
            self.base.parse()
        }
    }

    fn is_current_game_component(&self) -> bool {
        match self.base.current() {
            Token::Ident(ident) => matches!(ident.as_str(), "narrative" | "mechanics" | "assets" | "balance"),
            _ => false,
        }
    }

    fn is_current_game_token(&self) -> bool {
        match self.base.current() {
            Token::Ident(ident) => ident == "game",
            _ => false,
        }
    }

    fn parse_core_game_lambda(&mut self) -> Result<Expression, ParseError> {
        self.base.advance(); // consume 'game'

        // Parse the body expression
        let body = Box::new(self.base.parse()?);

        Ok(Expression::Literal(Literal::String(format!("λgame.{:?}", body))))
    }

    fn parse_component_lambda(&mut self) -> Result<Expression, ParseError> {
        let component_name = match self.base.current() {
            Token::Ident(ident) => ident.clone(),
            _ => return Err(ParseError::Expected("game component identifier".to_string(), self.base.current().clone())),
        };

        self.base.advance(); // consume component name

        // Parse the body expression
        let body = Box::new(self.base.parse()?);

        Ok(Expression::Literal(Literal::String(format!("λ{}.{:?}", component_name, body))))
    }

    /// Parse game rule: rule <name> = <condition> => <effect>
    pub fn parse_game_rule(&mut self) -> Result<GameExpression, ParseError> {
        self.base.expect(Token::Ident("rule".to_string()))?;

        let name = match self.base.advance() {
            Token::Ident(name) => name,
            token => return Err(ParseError::Expected("rule name".to_string(), token)),
        };

        self.base.expect(Token::Equals)?;

        // Parse condition (optional)
        let condition = if matches!(self.base.current(), Token::LParen) {
            self.base.advance();
            let cond = Box::new(self.base.parse()?);
            self.base.expect(crate::Token::RParen)?;
            Some(cond)
        } else {
            None
        };

        // Expect '=>'
        if !matches!(self.base.current(), Token::Ident(ref s) if s == "=>") {
            return Err(ParseError::Expected("=>".to_string(), self.base.current().clone()));
        }
        self.base.advance();

        // Parse effect
        let effect = Box::new(self.base.parse()?);

        Ok(GameExpression::GameRule { name, condition, effect })
    }

    /// Parse game transition: transition <from_state>? -> <to_state> [action]?
    pub fn parse_game_transition(&mut self) -> Result<GameExpression, ParseError> {
        self.base.expect(Token::Ident("transition".to_string()))?;

        // Parse from_state (optional)
        let from_state = if matches!(self.base.current(), Token::Ident(_)) {
            match self.base.advance() {
                Token::Ident(state) => Some(state),
                _ => return Err(ParseError::Expected("state identifier".to_string(), self.base.current().clone())),
            }
        } else {
            None
        };

        // Expect '->'
        if !matches!(self.base.current(), Token::Ident(ref s) if s == "->") {
            return Err(ParseError::Expected("->".to_string(), self.base.current().clone()));
        }
        self.base.advance();

        // Parse to_state
        let to_state = match self.base.advance() {
            Token::Ident(state) => state,
            token => return Err(ParseError::Expected("target state".to_string(), token)),
        };

        // Parse action (optional)
        let action = if matches!(self.base.current(), Token::LParen) {
            self.base.advance();
            let act = Box::new(self.base.parse()?);
            self.base.expect(crate::Token::RParen)?;
            Some(act)
        } else {
            None
        };

        Ok(GameExpression::GameTransition { from_state, to_state, action })
    }

    /// Parse balance expression: balance <score> [factor1, factor2, ...]
    pub fn parse_balance_expression(&mut self) -> Result<GameExpression, ParseError> {
        self.base.expect(Token::Ident("balance".to_string()))?;

        let score = match self.base.advance() {
            Token::Float(f) => f.parse::<f64>().unwrap_or(0.0),
            Token::Int(i) => i as f64,
            token => return Err(ParseError::Expected("balance score".to_string(), token)),
        };

        let mut factors = Vec::new();

        // Parse factors (optional)
        if matches!(self.base.current(), Token::LBracket) {
            self.base.advance();

            while !matches!(self.base.current(), Token::RBracket | Token::Eof) {
                let factor = self.parse_balance_factor()?;
                factors.push(factor);

                if matches!(self.base.current(), Token::Comma) {
                    self.base.advance();
                }
            }

            self.base.expect(crate::Token::RBracket)?;
        }

        Ok(GameExpression::BalanceExpr { score, factors })
    }

    fn parse_balance_factor(&mut self) -> Result<BalanceFactor, ParseError> {
        let name = match self.base.current() {
            Token::Ident(name) => name.clone(),
            _ => return Err(ParseError::Expected("factor name".to_string(), self.base.current().clone())),
        };

        self.base.advance();
        self.base.expect(Token::Colon)?;

        let weight = match self.base.advance() {
            Token::Float(f) => f.parse::<f64>().unwrap_or(1.0),
            Token::Int(i) => i as f64,
            token => return Err(ParseError::Expected("factor weight".to_string(), token)),
        };

        self.base.expect(Token::Equals)?;

        let expression = self.base.parse()?;

        Ok(BalanceFactor { name, weight, expression })
    }

    /// Parse asset specification: asset <type> { properties... }
    pub fn parse_asset_spec(&mut self) -> Result<GameExpression, ParseError> {
        self.base.expect(Token::Ident("asset".to_string()))?;

        let asset_type = match self.base.advance() {
            Token::Ident(type_name) => match type_name.as_str() {
                "sprite" => AssetType::Sprite,
                "sound" => AssetType::Sound,
                "music" => AssetType::Music,
                "texture" => AssetType::Texture,
                "model" => AssetType::Model,
                "animation" => AssetType::Animation,
                "ui" => AssetType::UI,
                _ => return Err(ParseError::Expected("valid asset type".to_string(), Token::Ident(type_name))),
            },
            token => return Err(ParseError::Expected("asset type".to_string(), token)),
        };

        let mut properties = HashMap::new();

        if matches!(self.base.current(), Token::LBrace) {
            self.base.advance();

            while !matches!(self.base.current(), Token::RBrace | Token::Eof) {
                let key = match self.base.advance() {
                    Token::Ident(key) => key,
                    token => return Err(ParseError::Expected("property name".to_string(), token)),
                };

                self.base.expect(Token::Colon)?;
                let value = self.base.parse()?;
                properties.insert(key, value);

                if matches!(self.base.current(), Token::Comma) {
                    self.base.advance();
                }
            }

            self.base.expect(crate::Token::RBrace)?;
        }

        Ok(GameExpression::AssetSpec { asset_type, properties })
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Parse a game expression string
pub fn parse_game_expression(input: &str) -> Result<Expression, ParseError> {
    let mut lexer = GameLexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = GameParser::new(tokens);

    // Try to parse as game lambda first
    parser.parse_game_lambda()
}

/// Parse a complex game manifest expression
pub fn parse_game_manifest(input: &str) -> Result<Vec<GameExpression>, ParseError> {
    let mut lexer = GameLexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = GameParser::new(tokens);
    let mut expressions = Vec::new();

    while !matches!(parser.base.current(), Token::Eof) {
        let expr = match parser.base.current() {
            Token::Ident(ref ident) => match ident.as_str() {
                "rule" => parser.parse_game_rule()?,
                "transition" => parser.parse_game_transition()?,
                "balance" => parser.parse_balance_expression()?,
                "asset" => parser.parse_asset_spec()?,
                "game" | "narrative" | "mechanics" | "assets" | "balance" => {
                    let base_expr = parser.parse_game_lambda()?;
                    // Convert to appropriate game expression
                    GameExpression::GameLambda {
                        body: Box::new(base_expr)
                    }
                }
                _ => {
                    let base_expr = parser.base.parse()?;
                    GameExpression::GameLambda {
                        body: Box::new(base_expr)
                    }
                }
            },
            _ => {
                let base_expr = parser.base.parse()?;
                GameExpression::GameLambda {
                    body: Box::new(base_expr)
                }
            }
        };

        expressions.push(expr);

        // Skip any commas between expressions
        if matches!(parser.base.current(), Token::Comma) {
            parser.base.advance();
        }
    }

    Ok(expressions)
}

/// Validate a game expression for proper structure
pub fn validate_game_expression(expr: &Expression) -> Result<(), String> {
    match expr {
        Expression::Literal(Literal::String(s)) => {
            if s.starts_with("λgame") || s.starts_with("λnarrative") ||
               s.starts_with("λmechanics") || s.starts_with("λassets") ||
               s.starts_with("λbalance") {
                // Basic structural validation
                if s.contains("λgame") {
                    validate_game_lambda_structure(s)
                } else if s.contains("λnarrative") {
                    validate_narrative_structure(s)
                } else if s.contains("λmechanics") {
                    validate_mechanics_structure(s)
                } else if s.contains("λassets") {
                    validate_assets_structure(s)
                } else if s.contains("λbalance") {
                    validate_balance_structure(s)
                } else {
                    Ok(())
                }
            } else {
                Err("Not a game expression".to_string())
            }
        }
        _ => Err("Expected string literal containing game expression".to_string()),
    }
}

fn validate_game_lambda_structure(expr: &str) -> Result<(), String> {
    if !expr.starts_with("λgame.") {
        return Err("Game lambda must start with λgame.".to_string());
    }

    // Check for balanced parentheses
    let mut paren_count = 0;
    for char in expr.chars() {
        match char {
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            _ => {}
        }
        if paren_count < 0 {
            return Err("Unbalanced parentheses in game expression".to_string());
        }
    }

    if paren_count != 0 {
        return Err("Unbalanced parentheses in game expression".to_string());
    }

    Ok(())
}

fn validate_narrative_structure(expr: &str) -> Result<(), String> {
    if !expr.starts_with("λnarrative.") {
        return Err("Narrative expression must start with λnarrative.".to_string());
    }
    Ok(())
}

fn validate_mechanics_structure(expr: &str) -> Result<(), String> {
    if !expr.starts_with("λmechanics.") {
        return Err("Mechanics expression must start with λmechanics.".to_string());
    }
    Ok(())
}

fn validate_assets_structure(expr: &str) -> Result<(), String> {
    if !expr.starts_with("λassets.") {
        return Err("Assets expression must start with λassets.".to_string());
    }
    Ok(())
}

fn validate_balance_structure(expr: &str) -> Result<(), String> {
    if !expr.starts_with("λbalance.") {
        return Err("Balance expression must start with λbalance.".to_string());
    }
    Ok(())
}

/// Convert game expression to canonical string representation
pub fn game_expression_to_string(expr: &GameExpression) -> String {
    match expr {
        GameExpression::GameLambda { body } => {
            format!("λgame.{}", body_to_string(body))
        }
        GameExpression::ComponentLambda { component, body } => {
            let comp_str = match component {
                GameComponent::Narrative => "narrative",
                GameComponent::Mechanics => "mechanics",
                GameComponent::Assets => "assets",
                GameComponent::Balance => "balance",
            };
            format!("λ{}.{}", comp_str, body_to_string(body))
        }
        GameExpression::GameRule { name, condition, effect } => {
            let cond_str = if let Some(cond) = condition {
                format!("({})", body_to_string(cond))
            } else {
                "true".to_string()
            };
            format!("rule {} = {} => {}", name, cond_str, body_to_string(effect))
        }
        GameExpression::GameTransition { from_state, to_state, action } => {
            let from_str = from_state.as_ref().map_or("".to_string(), |s| format!("{} -> ", s));
            let action_str = if let Some(act) = action {
                format!(" ({})", body_to_string(act))
            } else {
                "".to_string()
            };
            format!("transition {}{}{}", from_str, to_state, action_str)
        }
        GameExpression::BalanceExpr { score, factors } => {
            if factors.is_empty() {
                format!("balance {}", score)
            } else {
                let factor_strs: Vec<String> = factors.iter()
                    .map(|f| format!("{}: {} = {}", f.name, f.weight, body_to_string(&f.expression)))
                    .collect();
                format!("balance {} [{}]", score, factor_strs.join(", "))
            }
        }
        GameExpression::AssetSpec { asset_type, properties } => {
            let type_str = match asset_type {
                AssetType::Sprite => "sprite",
                AssetType::Sound => "sound",
                AssetType::Music => "music",
                AssetType::Texture => "texture",
                AssetType::Model => "model",
                AssetType::Animation => "animation",
                AssetType::UI => "ui",
            };

            if properties.is_empty() {
                format!("asset {}", type_str)
            } else {
                let prop_strs: Vec<String> = properties.iter()
                    .map(|(k, v)| format!("{}: {}", k, body_to_string(v)))
                    .collect();
                format!("asset {} {{{}}}", type_str, prop_strs.join(", "))
            }
        }
        GameExpression::NarrativeStructure { acts, choices } => {
            format!("narrative {{ acts: {:?}, choices: {:?} }}", acts, choices)
        }
        GameExpression::MechanicsSpec { turn_based, player_count, win_condition, lose_condition } => {
            format!("mechanics {{ turn_based: {:?}, players: {:?}, win: {:?}, lose: {:?} }}",
                turn_based, player_count, win_condition, lose_condition)
        }
    }
}

fn body_to_string(expr: &Expression) -> String {
    // Simple string conversion for expression body
    // In a real implementation, this would be more sophisticated
    format!("{:?}", expr)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_lambda_parsing() {
        let cases = vec![
            "λgame.(λnarrative.story=\"Space adventure\" λmechanics.turn_based=true)",
            "λnarrative.story=\"Space adventure\"",
            "λmechanics.turn_based=true",
            "λassets.sprite_size=32x32",
            "λbalance.score=0.85",
        ];

        for case in cases {
            let result = parse_game_expression(case);
            assert!(result.is_ok(), "Failed to parse: {}", case);
        }
    }

    #[test]
    fn test_game_rule_parsing() {
        // Note: This would require extending the parser to handle the full syntax
        // For now, we test the basic structure
        let expr = parse_game_expression("λgame.rule \"combat\" = (health > 0) => attack").unwrap();

        // Validate it's a proper game expression
        assert!(validate_game_expression(&expr).is_ok());
    }

    #[test]
    fn test_balance_expression_parsing() {
        let cases = vec![
            "λbalance.score=0.85",
            "λbalance.(score: 0.85, difficulty: 1.2)",
        ];

        for case in cases {
            let result = parse_game_expression(case);
            assert!(result.is_ok(), "Failed to parse balance expression: {}", case);
        }
    }

    #[test]
    fn test_asset_specification_parsing() {
        let cases = vec![
            "λassets.sprite { size: \"32x32\", format: \"png\" }",
            "λassets.sound { duration: \"2s\", format: \"wav\" }",
            "λassets.music { tempo: 120, key: \"C_major\" }",
        ];

        for case in cases {
            let result = parse_game_expression(case);
            assert!(result.is_ok(), "Failed to parse asset specification: {}", case);
        }
    }

    #[test]
    fn test_game_expression_validation() {
        let valid_cases = vec![
            "λgame.(λnarrative.story=\"test\" λmechanics.turn_based=true)",
            "λnarrative.story=\"test\"",
            "λmechanics.turn_based=true",
            "λassets.sprite_size=32x32",
            "λbalance.score=0.85",
        ];

        for case in valid_cases {
            let expr = parse_game_expression(case).unwrap();
            assert!(validate_game_expression(&expr).is_ok(),
                "Validation failed for: {}", case);
        }
    }

    #[test]
    fn test_complex_game_manifest_parsing() {
        let manifest = r#"
            λgame.(
                λnarrative.story="Space adventure with aliens"
                λmechanics.turn_based=true
                λassets.sprite_size=32x32
                λbalance.score=0.85
            )
        "#;

        let expressions = parse_game_manifest(manifest).unwrap();
        assert!(!expressions.is_empty(), "Should parse at least one expression");
    }

    #[test]
    fn test_malformed_expressions() {
        let invalid_cases = vec![
            "λgame.",  // Incomplete
            "game.story=\"test\"",  // Missing lambda
            "λnarrative",  // Incomplete
            "λinvalid.component",  // Invalid component
            "λgame.(unbalanced",  // Unbalanced parentheses
        ];

        for case in invalid_cases {
            let result = parse_game_expression(case);
            // Some might parse but fail validation
            if let Ok(expr) = result {
                assert!(validate_game_expression(&expr).is_err(),
                    "Should have failed validation: {}", case);
            }
        }
    }
}
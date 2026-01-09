// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Abstract Syntax Tree for .reqx expressions

use serde::{Deserialize, Serialize};

/// Expression AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// Literal value
    Literal(Literal),
    /// Variable reference {{name}}
    Variable(String),
    /// Path expression (e.g., res.body.data[0].id)
    Path(PathExpr),
    /// Binary operation
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    /// Function call (e.g., length, exists)
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    /// Pipe expression (e.g., res.body.data | length)
    Pipe {
        input: Box<Expression>,
        function: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathExpr {
    pub root: PathRoot,
    pub segments: Vec<PathSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathRoot {
    Res,      // Response
    Body,     // res.body shorthand
    Headers,  // res.headers shorthand
    Status,   // res.status shorthand
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathSegment {
    /// Property access (.name)
    Property(String),
    /// Index access ([0])
    Index(usize),
    /// Wildcard ([*])
    Wildcard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BinaryOperator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
    Contains,
    Matches,
}

impl BinaryOperator {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" | "==" => Some(Self::Equals),
            "!=" => Some(Self::NotEquals),
            "<" => Some(Self::LessThan),
            ">" => Some(Self::GreaterThan),
            "<=" => Some(Self::LessOrEqual),
            ">=" => Some(Self::GreaterOrEqual),
            "contains" => Some(Self::Contains),
            "matches" => Some(Self::Matches),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equals => "=",
            Self::NotEquals => "!=",
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::LessOrEqual => "<=",
            Self::GreaterOrEqual => ">=",
            Self::Contains => "contains",
            Self::Matches => "matches",
        }
    }
}

/// Built-in validation functions
#[derive(Debug, Clone, Copy)]
pub enum ValidationFunction {
    Exists,
    IsArray,
    IsNumber,
    IsString,
    IsUuid,
    IsIso8601,
    IsEmail,
}

impl ValidationFunction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "exists" => Some(Self::Exists),
            "is_array" => Some(Self::IsArray),
            "is_number" => Some(Self::IsNumber),
            "is_string" => Some(Self::IsString),
            "is_uuid" => Some(Self::IsUuid),
            "is_iso8601" => Some(Self::IsIso8601),
            "is_email" => Some(Self::IsEmail),
            _ => None,
        }
    }
}

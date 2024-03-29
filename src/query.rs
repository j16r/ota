use std::convert::TryFrom;

use anyhow::Result;
use thiserror::Error;

#[derive(Clone, Debug, Default)]
pub struct Query {
    pub id: Option<String>,
    pub properties: Vec<PropertyFilter>,
    pub tags: Vec<String>,
}

pub const ALL: &Query = &Query {
    id: None,
    properties: vec![],
    tags: vec![],
};

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyFilter {
    field: String,
    operator: PropertyOperator,
}

#[derive(Clone, Debug, PartialEq)]
enum PropertyOperator {
    Equals(String),
    Lt(String),
    Gt(String),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum QueryParseError {
    #[error("missing right hand side of property filter")]
    MissingOperatorArgument,
    #[error("missing left hand side of property filter")]
    MissingOperatorField,
    #[error("duplicate id filter")]
    DuplicateID,
}

impl<'a> TryFrom<&'a str> for Query {
    type Error = QueryParseError;

    fn try_from(query: &'a str) -> Result<Self, Self::Error> {
        let mut result = Query::default();
        let operators: &[_] = &['=', '<', '>'];

        for capture in query.split_whitespace() {
            if let Some(id) = capture.strip_prefix('@') {
                if result.id.is_none() {
                    result.id = Some(id.into());
                } else {
                    return Err(QueryParseError::DuplicateID);
                }
            } else if let Some(pos) = capture.find(operators) {
                let (field, operator_and_arg) = capture.split_at(pos);

                if field.is_empty() {
                    return Err(QueryParseError::MissingOperatorArgument);
                } else if operator_and_arg.len() < 2 {
                    return Err(QueryParseError::MissingOperatorField);
                }

                let (operator, argument) = operator_and_arg.split_at(1);
                result.properties.push(PropertyFilter {
                    field: field.to_string(),
                    operator: match operator {
                        "=" => PropertyOperator::Equals(argument.to_string()),
                        ">" => PropertyOperator::Gt(argument.to_string()),
                        "<" => PropertyOperator::Lt(argument.to_string()),
                        _ => unreachable!(),
                    },
                });
            } else {
                result.tags.push(capture.into());
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::query::*;
    use std::convert::TryInto;

    #[test]
    fn test_query_from_str() {
        let mut query: Query = "@index".try_into().unwrap();
        assert_eq!(query.id, Some("index".to_string()));

        query = "tag".try_into().unwrap();
        assert_eq!(query.tags, vec!["tag".to_string()]);

        query = "tag1 tag2".try_into().unwrap();
        assert_eq!(query.tags, vec!["tag1".to_string(), "tag2".to_string()]);

        query = "count=1".try_into().unwrap();
        assert_eq!(
            query.properties,
            vec![PropertyFilter {
                field: "count".to_string(),
                operator: PropertyOperator::Equals("1".to_string()),
            }]
        );
    }

    #[test]
    fn test_query_from_str_invalid() {
        let mut query: Result<Query, _> = "@index @index".try_into();
        assert!(query.is_err());
        assert_eq!(query.unwrap_err(), QueryParseError::DuplicateID);

        query = "count=".try_into();
        assert!(query.is_err());
        assert_eq!(query.unwrap_err(), QueryParseError::MissingOperatorField);

        query = "=".try_into();
        assert!(query.is_err());
        assert_eq!(query.unwrap_err(), QueryParseError::MissingOperatorArgument);
    }
}

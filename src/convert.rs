/*
 * Copyright 2020, Ulf Lilleengen
 * License: Apache License 2.0 (see the file LICENSE or http://apache.org/licenses/LICENSE-2.0.html).
 */

use std::collections::BTreeMap;
use std::vec::Vec;

use crate::error::*;
use crate::frame_codec::*;
use crate::symbol::*;
use crate::types::*;

/**
 *************************************************************************************
 * Conversion functions for types that is used in this implementation. This is used  *
 * when decoding frames. At present there is a lot of duplication, and this part     *
 * could use some refactoring to simplify.                                           *
 *************************************************************************************
 */

pub trait TryFromValue {
    fn try_from(value: Value) -> Result<Self>
    where
        Self: std::marker::Sized;
}

impl<T: TryFromValue> TryFromValue for Option<T> {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Null => Ok(None),
            _ => Ok(Some(T::try_from(value)?)),
        }
    }
}

impl<T: TryFromValue> TryFromValue for Vec<T> {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(v) => {
                let (results, errors): (Vec<_>, Vec<_>) = v
                    .into_iter()
                    .map(|f| T::try_from(f))
                    .partition(Result::is_ok);
                if errors.len() > 0 {
                    return Err(AmqpError::decode_error(Some(
                        "Error decoding list elements",
                    )));
                } else {
                    return Ok(results.into_iter().map(Result::unwrap).collect());
                }
            }
            Value::Array(v) => {
                let (results, errors): (Vec<_>, Vec<_>) = v
                    .into_iter()
                    .map(|f| T::try_from(f))
                    .partition(Result::is_ok);
                if errors.len() > 0 {
                    return Err(AmqpError::decode_error(Some(
                        "Error decoding array elements",
                    )));
                } else {
                    return Ok(results.into_iter().map(Result::unwrap).collect());
                }
            }
            _ => return Ok(vec![T::try_from(value)?]),
        }
    }
}

impl TryFromValue for u8 {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Ubyte(v) => return Ok(v),
            _ => Err(AmqpError::amqp_error(
                condition::DECODE_ERROR,
                Some("Error converting value to u8"),
            )),
        }
    }
}

impl TryFromValue for u64 {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Ulong(v) => return Ok(v),
            _ => Err(AmqpError::decode_error(Some(
                "Error converting value to u64",
            ))),
        }
    }
}

impl TryFromValue for u32 {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Uint(v) => return Ok(v),
            _ => Err(AmqpError::amqp_error(
                condition::DECODE_ERROR,
                Some("Error converting value to u32"),
            )),
        }
    }
}

impl TryFromValue for u16 {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Ushort(v) => return Ok(v),
            _ => Err(AmqpError::amqp_error(
                condition::DECODE_ERROR,
                Some("Error converting value to u32"),
            )),
        }
    }
}

impl TryFromValue for bool {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Bool(v) => return Ok(v),
            _ => Err(AmqpError::decode_error(Some(
                "Error converting value to bool",
            ))),
        }
    }
}

impl TryFromValue for String {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Symbol(v) => Ok(String::from_utf8_lossy(&v[..]).to_string()),
            Value::String(v) => Ok(v),
            _ => Err(AmqpError::amqp_error(
                condition::DECODE_ERROR,
                Some("Error converting value to String"),
            )),
        }
    }
}

impl TryFromValue for BTreeMap<String, Value> {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Map(v) => {
                let mut m = BTreeMap::new();
                for (key, value) in v.into_iter() {
                    m.insert(String::try_from(key)?, value);
                }
                Ok(m)
            }
            _ => Err(AmqpError::amqp_error(
                condition::DECODE_ERROR,
                Some("Error converting value to Vec<Symbol>"),
            )),
        }
    }
}

impl TryFromValue for BTreeMap<Value, Value> {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Map(v) => Ok(v),
            _ => Err(AmqpError::amqp_error(
                condition::DECODE_ERROR,
                Some("Error converting value to Vec<Symbol>"),
            )),
        }
    }
}

impl TryFromValue for Symbol {
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Symbol(v) => Ok(Symbol::from_vec(v)),
            _ => Err(AmqpError::amqp_error(
                condition::DECODE_ERROR,
                Some("Error converting value to Symbol"),
            )),
        }
    }
}

impl TryFromValue for ErrorCondition {
    fn try_from(value: Value) -> Result<Self> {
        if let Value::Described(descriptor, mut list) = value {
            let decoder = FrameDecoder::new(&descriptor, &mut list)?;
            match *descriptor {
                DESC_ERROR => ErrorCondition::decode(decoder),
                _ => Err(AmqpError::decode_error(Some(
                    format!("Expected error descriptor but found {:?}", *descriptor).as_str(),
                ))),
            }
        } else {
            Err(AmqpError::decode_error(Some(
                "Missing expected error descriptor",
            )))
        }
    }
}

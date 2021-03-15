// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tuple param type.

use crate::ParamType;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use serde::{
	de::{Error, MapAccess, Visitor},
	Deserialize, Deserializer,
};
#[cfg(feature = "std")]
use std::fmt;

/// Tuple params specification
#[derive(Debug, Clone, PartialEq)]
pub struct TupleParam {
	/// Param name.
	pub name: Option<String>,

	/// Param type.
	pub kind: ParamType,
}

#[cfg(feature = "std")]
impl<'a> Deserialize<'a> for TupleParam {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_any(TupleParamVisitor)
	}
}

#[cfg(feature = "std")]
struct TupleParamVisitor;

#[cfg(feature = "std")]
impl<'a> Visitor<'a> for TupleParamVisitor {
	type Value = TupleParam;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid tuple parameter spec")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'a>,
	{
		let mut name = None;
		let mut kind = None;
		let mut components = None;

		while let Some(ref key) = map.next_key::<String>()? {
			match key.as_ref() {
				"name" => {
					if name.is_some() {
						return Err(Error::duplicate_field("name"));
					}
					name = Some(map.next_value()?);
				}
				"type" => {
					if kind.is_some() {
						return Err(Error::duplicate_field("type"));
					}
					kind = Some(map.next_value()?);
				}
				"components" => {
					if components.is_some() {
						return Err(Error::duplicate_field("components"));
					}
					let component: Vec<TupleParam> = map.next_value()?;
					components = Some(component)
				}
				_ => {}
			}
		}

		let kind = kind.ok_or_else(|| Error::missing_field("kind")).and_then(|param_type| {
			if let ParamType::Tuple(_) = param_type {
				let tuple_params = components.ok_or_else(|| Error::missing_field("components"))?;
				Ok(ParamType::Tuple(tuple_params.into_iter().map(|param| param.kind).collect()))
			} else {
				Ok(param_type)
			}
		})?;

		Ok(TupleParam { name, kind })
	}
}

#[cfg(all(test, feature = "std"))]
mod tests {
	use crate::{ParamType, TupleParam};

	#[test]
	fn tuple_param_deserialization() {
		let s = r#"[{
			"name": "foo",
			"type": "address"
			},{
			"name": "bar",
			"type": "address"
			},{
			"name": "baz",
			"type": "address"
			},{
			"type": "bool"
			}
		]"#;

		let deserialized: Vec<TupleParam> = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			vec![
				TupleParam { name: Some(String::from("foo")), kind: ParamType::Address },
				TupleParam { name: Some(String::from("bar")), kind: ParamType::Address },
				TupleParam { name: Some(String::from("baz")), kind: ParamType::Address },
				TupleParam { name: None, kind: ParamType::Bool },
			]
		);
	}
}

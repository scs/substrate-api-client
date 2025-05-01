/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

	   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

use alloc::{
	format,
	string::{String, ToString},
};
use serde_json::Value;

pub fn read_subscription_id(value: &Value) -> Option<String> {
	value["result"].as_str().map(|str| str.to_string())
}

pub fn read_error_message(value: &Value, msg: &str) -> String {
	match value["error"].as_str() {
		Some(error_message) => error_message.to_string(),
		None => format!("Unexpected Response: {}", msg),
	}
}

pub fn subscription_id_matches(value: &Value, subscription_id: &str) -> bool {
	match value["params"]["subscription"].as_str() {
		Some(retrieved_subscription_id) => subscription_id == retrieved_subscription_id,
		None => false,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn read_valid_subscription_response() {
		let subcription_id = "tejkataa12124a";
		let value = json!({
			"result": subcription_id,
			"id": 43,
			"and_so_on": "test",
		});

		let maybe_subcription_id = read_subscription_id(&value);
		assert_eq!(maybe_subcription_id, Some(subcription_id.to_string()));
	}

	#[test]
	fn read_invalid_subscription_response() {
		let subcription_id = "tejkataa12124a";
		let value = json!({
			"error": subcription_id,
			"id": 43,
			"and_so_on": "test",
		});

		let maybe_subcription_id = read_subscription_id(&value);
		assert!(maybe_subcription_id.is_none());
	}

	#[test]
	fn read_error_message_returns_error_if_available() {
		let error_message = "some_error_message";
		let value = json!({
			"error": error_message,
			"id": 43,
			"and_so_on": "test",
		});

		let msg = serde_json::to_string(&value).unwrap();

		let message = read_error_message(&value, &msg);
		assert!(message.contains(error_message));
		assert!(message.contains("error"));
	}

	#[test]
	fn read_error_message_returns_full_msg_if_error_is_not_available() {
		let error_message = "some_error_message";
		let value = json!({
			"result": error_message,
			"id": 43,
			"and_so_on": "test",
		});

		let msg = serde_json::to_string(&value).unwrap();

		let message = read_error_message(&value, &msg);
		assert!(message.contains(&msg));
	}

	#[test]
	fn subscription_id_matches_returns_true_for_equal_id() {
		let subcription_id = "tejkataa12124a";
		let value = json!({
			"params": {
				"subscription": subcription_id,
				"message": "Test"
			},
			"id": 43,
			"and_so_on": "test",
		});

		assert!(subscription_id_matches(&value, subcription_id));
	}

	#[test]
	fn subscription_id_matches_returns_false_for_not_equal_id() {
		let subcription_id = "tejkataa12124a";
		let value = json!({
			"params": {
				"subscription": "something else",
				"message": "Test"
			},
			"id": 43,
			"and_so_on": "test",
		});

		assert!(!subscription_id_matches(&value, subcription_id));
	}

	#[test]
	fn subscription_id_matches_returns_false_for_missing_subscription() {
		let subcription_id = "tejkataa12124a";
		let value = json!({
			"params": {
				"result": subcription_id,
				"message": "Test"
			},
			"id": 43,
			"and_so_on": "test",
		});

		assert!(!subscription_id_matches(&value, subcription_id));
	}
}

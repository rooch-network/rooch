use cosmwasm_std::{
  to_json_binary, Binary, Deps, Env, StdResult, Response, MessageInfo, DepsMut,
  entry_point,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  InscribeGenerate(InputData),
  InscribeVerify { input: InputData, output: OutputData },
  IndexerGenerate(InputData),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InputData {
  seed: String,
  user_input: String,
  attributes: Vec<AttributeGenerateRule>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AttributeGenerateRule {
  key: String,
  rule_type: AttributeRuleType,
  parameters: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AttributeRuleType {
  Range,
  List,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OutputData {
  amount: u32,
  attributes: Vec<Attribute>,
  content: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Attribute {
  key: String,
  value: String,
}


#[entry_point]
pub fn instantiate(
  _deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  _msg: InstantiateMsg,
) -> StdResult<Response> {
  Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
  match msg {
      QueryMsg::InscribeGenerate(input) => to_json_binary(&query_inscribe_generate(deps, input)?),
      QueryMsg::InscribeVerify { input, output } => to_json_binary(&inscribe_verify(deps, input, output)?),
      QueryMsg::IndexerGenerate(input) => to_json_binary(&indexer_generate(deps, input)?),
  }
}

fn query_inscribe_generate(_deps: Deps, input: InputData) -> StdResult<OutputData> {
  let hash_value = hash_str_uint32(&format!("{}{}", input.seed, input.user_input));

  let mut attributes = vec![
      Attribute {
          key: "id".to_string(),
          value: input.user_input.clone(),
      }
  ];

  for attr in input.attributes {
      let value = match attr.rule_type {
          AttributeRuleType::Range => {
              if attr.parameters.len() == 2 {
                  if let (Ok(min), Ok(max)) = (attr.parameters[0].parse::<u32>(), attr.parameters[1].parse::<u32>()) {
                      let random_value = min + (hash_value % (max - min + 1));
                      random_value.to_string()
                  } else {
                      continue; // Skip this attribute if parsing fails
                  }
              } else {
                  continue; // Skip this attribute if parameters are incorrect
              }
          },
          AttributeRuleType::List => {
              if !attr.parameters.is_empty() {
                  let index = (hash_value as usize) % attr.parameters.len();
                  attr.parameters[index].clone()
              } else {
                  continue; // Skip this attribute if the list is empty
              }
          },
      };

      attributes.push(Attribute {
          key: attr.key,
          value,
      });
  }

  Ok(OutputData {
      amount: 1000,
      attributes,
      content: Binary::default(),
  })
}

fn inscribe_verify(deps: Deps, input: InputData, output: OutputData) -> StdResult<bool> {
  let generated_output = query_inscribe_generate(deps, input)?;
  Ok(generated_output == output)
}

fn indexer_generate(deps: Deps, input: InputData) -> StdResult<OutputData> {
  query_inscribe_generate(deps, input)
}

fn hash_str_uint32(str: &str) -> u32 {
  let mut hash: u32 = 0x811c9dc5;
  let prime: u32 = 0x1000193;

  for &byte in str.as_bytes() {
      hash ^= u32::from(byte);
      hash = hash.wrapping_mul(prime);
  }

  hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::from_json;

    #[test]
    fn test_inscribe_generate() {
        // Step 1: Set up the test environment
        let deps = mock_dependencies();
        let env = mock_env();

        // Step 2: Prepare test data
        let input = InputData {
            seed: "test_seed".to_string(),
            user_input: "test_user_input".to_string(),
            attributes: vec![
                AttributeGenerateRule {
                    key: "test_range".to_string(),
                    rule_type: AttributeRuleType::Range,
                    parameters: vec!["1".to_string(), "10".to_string()],
                },
                AttributeGenerateRule {
                    key: "test_list".to_string(),
                    rule_type: AttributeRuleType::List,
                    parameters: vec!["A".to_string(), "B".to_string(), "C".to_string()],
                },
            ],
        };

        // Step 3: Call the query function
        let query_msg = QueryMsg::InscribeGenerate(input);
        let binary_response = query(deps.as_ref(), env, query_msg).unwrap();

        // Step 4: Parse the response
        let output: OutputData = from_json(&binary_response).unwrap();

        // Step 5: Validate the results
        assert_eq!(output.amount, 1000);
        assert_eq!(output.attributes.len(), 3); // id + test_range + test_list

        // Validate id attribute
        assert_eq!(output.attributes[0].key, "id");
        assert_eq!(output.attributes[0].value, "test_user_input");

        // Validate test_range attribute
        assert_eq!(output.attributes[1].key, "test_range");
        let range_value: u32 = output.attributes[1].value.parse().unwrap();
        assert!(range_value >= 1 && range_value <= 10);

        // Validate test_list attribute
        assert_eq!(output.attributes[2].key, "test_list");
        assert!(["A", "B", "C"].contains(&output.attributes[2].value.as_str()));

        // Verify that content is empty
        assert_eq!(output.content, Binary::default());
    }

    #[test]
    fn test_inscribe_verify() {
        // Set up dependencies and environment
        let deps = mock_dependencies();
        let env = mock_env();

        // Prepare test data
        let input = InputData {
            seed: "test_seed".to_string(),
            user_input: "test_user_input".to_string(),
            attributes: vec![
                AttributeGenerateRule {
                    key: "test_range".to_string(),
                    rule_type: AttributeRuleType::Range,
                    parameters: vec!["1".to_string(), "10".to_string()],
                },
            ],
        };

        // Generate expected output
        let expected_output = query_inscribe_generate(deps.as_ref(), input.clone()).unwrap();

        // Test verification with correct output
        let query_msg = QueryMsg::InscribeVerify {
            input: input.clone(),
            output: expected_output.clone(),
        };
        let binary_response = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let result: bool = from_json(&binary_response).unwrap();
        assert!(result);

        // Test verification with incorrect output
        let incorrect_output = OutputData {
            amount: 999, // Incorrect amount
            ..expected_output
        };
        let query_msg = QueryMsg::InscribeVerify {
            input,
            output: incorrect_output,
        };
        let binary_response = query(deps.as_ref(), env, query_msg).unwrap();
        let result: bool = from_json(&binary_response).unwrap();
        assert!(!result);
    }
    
}

use std::collections::HashMap;

pub fn evaluate(original: &str, variables: &HashMap<String, String>) -> String {
  original.split(' ').map(|original_part|
      match (*variables).get(original_part) {
          Some(v) => &v[..],
          None => original_part,
      }
  ).collect::<Vec<_>>().connect(" ")
}

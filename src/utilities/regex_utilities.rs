/*
 Copyright (c) 2023 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use crate::models::matches::Match;
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use tree_sitter::Node;

/// Applies the query upon the given `node`, and gets the first match
/// # Arguments
/// * `node` - the root node to apply the query upon
/// * `source_code` - the corresponding source code string for the node.
/// * `query` - the query to be applied
/// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
///
/// # Returns
/// The range of the match in the source code and the corresponding mapping from tags to code snippets.
pub(crate) fn get_all_matches_for_regex(
  node: &Node, source_code: String, regex: &Regex, recursive: bool, replace_node: Option<String>,
) -> Vec<Match> {
  let code_snippet = node.utf8_text(source_code.as_bytes()).unwrap();
  let all_captures = regex.captures_iter(code_snippet).collect_vec();
  let names = regex.capture_names().collect_vec();
  let mut all_matches = vec![];
  for captures in all_captures {
    // Check if the range of the self (node), and the range of outermost node captured by the query are equal.
    let range_matches_node = node.start_byte() == captures.get(0).unwrap().start()
      && node.end_byte() == captures.get(0).unwrap().end();
    if recursive || range_matches_node {
      let group_by_tag = if let Some(ref rn) = replace_node {
        captures
          .name(rn)
          .unwrap_or_else(|| panic!("the tag {rn} provided in the replace node is not present"))
      } else {
        captures.get(0).unwrap()
      };
      let matches = extract_captures(&captures, &names);
      all_matches.push(Match::from_regex(&group_by_tag, matches, code_snippet));
    }
  }
  all_matches
}

fn extract_captures(
  captures: &regex::Captures<'_>, names: &Vec<Option<&str>>,
) -> HashMap<String, String> {
  names
    .iter()
    .flatten()
    .flat_map(|x| {
      captures
        .name(x)
        .map(|v| (x.to_string(), v.as_str().to_string()))
    })
    .collect()
}

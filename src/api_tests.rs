use super::*;

fn encode_pair(pair: Pair, use_base64: bool) -> String {
  match pair {
    Pair(key, val) => {
      let encoded_val = if use_base64 {
        base64::encode(&val)
      } else {
        val
      };
      format!("{key}={encoded_val}")
    }
  }
}

#[test]
fn test_extract_key_from_path() {
  assert_eq!(
    extract_key_from_path("/app/env/DATABASE_URL").unwrap(),
    "DATABASE_URL"
  );
  assert_eq!(extract_key_from_path("/app/env/foo").unwrap(), "foo");
  assert_eq!(
    extract_key_from_path("/app/foo").unwrap_err().to_string(),
    Error::InvalidPathError(String::from("Invalid path /app/foo")).to_string()
  );
  assert_eq!(
    extract_key_from_path("/app/foo/bar/car")
      .unwrap_err()
      .to_string(),
    Error::InvalidPathError(String::from("Invalid path /app/foo/bar/car")).to_string()
  );
}

#[test]
fn test_as_env_format() {
  let map: HashMap<String, String> = vec![
    ("one".to_owned(), "bar".to_owned()),
    ("two".to_owned(), "baz".to_owned()),
    ("THREE".to_owned(), "clock".to_owned()),
  ]
  .into_iter()
  .collect();
  let env_format = as_env_format(map, true);
  let mut result: Vec<&str> = env_format.trim().split("\n").collect();
  result.sort_by_key(|line| line.to_lowercase());
  assert_eq!(result, vec!["one=bar", "THREE=clock", "two=baz"]);
}

#[test]
fn test_escape_for_bash() {
  assert_eq!(escape_for_bash(r#"a$`"\'!)&"#), r#"a\$\`\"\\'\!\)&"#);
}

#[test]
fn test_read_from_reader() {
  let pair1 = encode_pair(Pair("foo".to_owned(), "bar".to_owned()), false);
  let pair2 = encode_pair(Pair("baz".to_owned(), "qux".to_owned()), false);
  let source = format!("{pair1}\n{pair2}\n").into_bytes();
  let result = read_from_reader(BufReader::new(Cursor::new(source)), false).unwrap();
  let expected: HashMap<String, String> = vec![
    ("foo".to_owned(), "bar".to_owned()),
    ("baz".to_owned(), "qux".to_owned()),
  ]
  .into_iter()
  .collect();
  assert_eq!(result, expected);
}

#[test]
fn test_read_from_reader_as_base64() {
  let pair1 = encode_pair(Pair("foo".to_owned(), "bar".to_owned()), true);
  let pair2 = encode_pair(Pair("baz".to_owned(), "qux".to_owned()), true);
  let source = format!("{pair1}\n{pair2}\n").into_bytes();
  let result = read_from_reader(BufReader::new(Cursor::new(source)), true).unwrap();
  let expected: HashMap<String, String> = vec![
    ("foo".to_owned(), "bar".to_owned()),
    ("baz".to_owned(), "qux".to_owned()),
  ]
  .into_iter()
  .collect();
  assert_eq!(result, expected);
}

#[test]
fn test_read_from_reader_with_extra_lines() {
  let pair1 = encode_pair(Pair("foo".to_owned(), "bar".to_owned()), true);
  let pair2 = encode_pair(Pair("baz".to_owned(), "qux".to_owned()), true);
  let source = format!("{pair1}\n\r\n\n{pair2}\n\n").into_bytes();
  let result = read_from_reader(BufReader::new(Cursor::new(source)), true).unwrap();
  let expected: HashMap<String, String> = vec![
    ("foo".to_owned(), "bar".to_owned()),
    ("baz".to_owned(), "qux".to_owned()),
  ]
  .into_iter()
  .collect();
  assert_eq!(result, expected);
}

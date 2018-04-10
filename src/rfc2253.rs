/**
 * Copyright © 2018 Nyantec GmbH <oss@nyantec.com>
 * Authors:
 *   Paul Asmuth <asm@nyantec.com>
 *
 * Provided that these terms and disclaimer and all copyright notices
 * are retained or reproduced in an accompanying document, permission
 * is granted to deal in this work without restriction, including un‐
 * limited rights to use, publicly perform, distribute, sell, modify,
 * merge, give away, or sublicence.
 *
 * This work is provided “AS IS” and WITHOUT WARRANTY of any kind, to
 * the utmost extent permitted by applicable law, neither express nor
 * implied; without malicious intent or gross negligence. In no event
 * may a licensor, author or contributor be held liable for indirect,
 * direct, other damage, loss, or other issues arising in any way out
 * of dealing in the work, even if advised of the possibility of such
 * damage or existence of a defect, except proven that it results out
 * of said person’s immediate fault when using the work as intended.
 */
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DistinguishedName {
	pub common_name: Option<String>,
	pub attributes: HashMap<String, String>
}

impl DistinguishedName {

	pub fn from_common_name(name: &str) -> DistinguishedName {
		return DistinguishedName {
			common_name: Some(name.to_owned()),
			attributes: HashMap::<String, String>::new()
		}
	}

	pub fn new() -> DistinguishedName {
		return DistinguishedName {
			common_name: None,
			attributes: HashMap::<String, String>::new()
		}
	}

	pub fn get_email(self: &DistinguishedName) -> Option<String> {
		return self.attributes.get("emailAddress").map(|s| s.to_owned());
	}

}

struct ParserState<'a> {
	input: &'a [char],
	cur: usize
}

impl<'a> ParserState<'a> {

	pub fn get(self: &ParserState<'a>) -> Option<char> {
		if self.cur < self.input.len() {
			return Some(self.input[self.cur]);
		} else {
			return None;
		}
	}

	pub fn eof(self: &ParserState<'a>) -> bool {
		return self.cur >= self.input.len();
	}

	pub fn consume(self: &mut ParserState<'a>) {
		self.cur += 1;
	}

}

struct Parser<'a> {
	state: ParserState<'a>,
}

impl<'a> Parser<'a> {

	pub fn new_with_input(input: &[char]) -> Parser{
		return Parser {
			state: ParserState {
				input: input,
				cur: 0
			}
		}
	}

	pub fn parse_attributes(self: &mut Parser<'a>) -> Result<HashMap<String, String>, ()> {
		let mut map = HashMap::<String, String>::new();

		loop {
			if self.state.eof() {
				break
			}

			match self.parse_attribute() {
				Ok(Some((k, v))) => { map.insert(k, v); },
				Ok(None) => return Err(()),
				Err(_) => return Err(())
			}

			match self.state.get() {
				Some(',') => { self.state.consume(); },
				Some('+') => { self.state.consume(); },
				Some(_) => return Err(()),
				None => break,
			}
		}

		return Ok(map);
	}

	pub fn parse_attribute(self: &mut Parser<'a>) -> Result<Option<(String, String)>, ()> {
		let attr_type = match self.parse_attribute_type() {
			Ok(s) => s,
			Err(_) => return Err(())
		};

		match self.state.get() {
			Some('=') => { self.state.consume(); },
			Some(_) => return Err(()),
			None => return Err(()),
		}

		let attr_value = match self.parse_attribute_value() {
			Ok(s) => s,
			Err(_) => return Err(())
		};

		return Ok(Some((attr_type, attr_value)));
	}

	fn parse_attribute_type(self: &mut Parser<'a>) -> Result<String, ()> {
		match self.state.get() {
			Some(c) =>
				if is_alpha(c) {
					return self.parse_attribute_type_keychar();
				} else if is_digit(c) {
					return self.parse_attribute_type_oid();
				} else {
					return Err(());
				},
			None => return Err(()),
		}
	}

	fn parse_attribute_type_keychar(self: &mut Parser<'a>) -> Result<String, ()> {
		let mut buf = String::new();

		loop {
			match self.state.get() {
				Some(c) =>
					if is_alpha(c) || is_digit(c) || c == '-' {
						buf.push(c);
						self.state.consume();
					} else{
						break;
					},
				None => return Err(()),
			}
		}

		return Ok(buf);
	}

	fn parse_attribute_type_oid(self: &mut Parser<'a>) -> Result<String, ()> {
		let mut buf = String::new();

		loop {
			match self.state.get() {
				Some(c) =>
					if is_digit(c) || c == '.' {
						buf.push(c);
						self.state.consume();
					} else{
						break;
					},
				None => return Err(()),
			}
		}

		return Ok(buf);
	}

	fn parse_attribute_value(self: &mut Parser<'a>) -> Result<String, ()> {
		return self.parse_string();
	}

	fn parse_string(self: &mut Parser<'a>) -> Result<String, ()> {
		return match self.state.get() {
			Some('"') => return self.parse_string_quoted(),
			Some('#') => return self.parse_hexstring(), 
			Some(_) => return self.parse_string_simple(),
			None => Err(()),
		}
	}

	fn parse_string_simple(self: &mut Parser<'a>) -> Result<String, ()> {
		let mut buf = String::new();

		loop {
			match self.state.get() {
				Some(c) =>
					if is_special(c) {
						break;
					} else if is_quotation(c) {
						return Err(());
					} else if is_escape(c) {
						match self.parse_escape_sequence() {
							Ok(s) => buf += &s,
							Err(_) => return Err(())
						}
					} else {
						buf.push(c);
						self.state.consume();
					},
				None => break,
			}
		}

		return Ok(buf);
	}

	fn parse_string_quoted(self: &mut Parser<'a>) -> Result<String, ()> {
		let mut buf = String::new();

		match self.state.get() {
			Some('"') => { self.state.consume(); },
			Some(_) => return Err(()),
			None => return Err(()),
		}

		loop {
			match self.state.get() {
				Some(c) =>
					if is_quotation(c) {
						break;
					} else if is_escape(c) {
						match self.parse_escape_sequence() {
							Ok(s) => buf += &s,
							Err(_) => return Err(())
						}
					} else {
						buf.push(c);
						self.state.consume();
					},
				None => return Err(()),
			}
		}

		match self.state.get() {
			Some('"') => { self.state.consume(); },
			Some(_) => return Err(()),
			None => return Err(()),
		}

		return Ok(buf);
	}

	fn parse_escape_sequence(self: &mut Parser<'a>) -> Result<String, ()> {
		match self.state.get() {
			Some('\\') => { self.state.consume(); },
			Some(_) => return Err(()),
			None => return Err(()),
		}

		return match self.state.get() {
			Some(c) =>
					if is_special(c) || is_quotation(c) || is_escape(c) {
						self.state.consume();
						Ok(c.to_string())
					} else if is_hexchar(c) {
						return self.parse_hexpair();
					} else {
						Err(())
					},
			None => Err(()),
		}
	}

	fn parse_hexstring(self: &mut Parser<'a>) -> Result<String, ()> {
		match self.state.get() {
			Some('#') => { self.state.consume(); },
			Some(_) => return Err(()),
			None => return Err(()),
		}

		let mut buf = String::new();

		loop {
			match self.state.get() {
				Some(c) =>
					if is_hexchar(c) {
						match self.parse_hexpair() {
							Ok(s) => buf += &s,
							Err(_) => return Err(()),
						}
					} else {
						return Err(());
					},
				None => break,
			}
		}

		return Ok(buf);
	}

	fn parse_hexpair(self: &mut Parser<'a>) -> Result<String, ()> {
		let mut buf = String::new();

		for _ in 0..2 {
			match self.state.get() {
				Some(c) => { buf.push(c); self.state.consume(); },
				None => return Err(()),
			};
		}

		match u32::from_str_radix(&buf, 16) {
			Ok(i) =>
				match std::char::from_u32(i) {
					Some(c) => return Ok(c.to_string()),
					None => return Err(())
				},
			Err(_) => return Err(())
		};
	}

}

pub fn is_alpha(chr: char) -> bool {
	return (chr >= 'A' && chr <= 'Z') || (chr >= 'a' && chr <= 'z');
}

pub fn is_digit(chr: char) -> bool {
	return chr >= '0' && chr <= '9';
}

pub fn is_special(chr: char) -> bool {
	return
			chr == ',' ||
			chr == '=' ||
			chr == '+' ||
			chr == '<' ||
			chr == '>' ||
			chr == '#' ||
			chr == ';';
}

pub fn is_quotation(chr: char) -> bool {
	return chr == '"';
}

pub fn is_escape(chr: char) -> bool {
	return chr == '\\';
}

pub fn is_hexchar(chr: char) -> bool {
	return
			(chr >= '0' && chr <= '9') ||
			(chr >= 'A' && chr <= 'F') ||
			(chr >= 'a' && chr <= 'f');
}

pub fn parse_distinguished_name(input: &[char]) -> Result<DistinguishedName, ()> {
	let mut parser = Parser::new_with_input(input);

	return match parser.parse_attributes() {
		Ok(attr) => Ok(DistinguishedName {
			common_name: attr.get("CN").map(|x| x.to_owned()),
			attributes: attr
		}),
		Err(_) => Err(()),
	}
}

pub fn parse_distinguished_name_str(input: &str) -> Result<DistinguishedName, ()> {
	let chars = input.chars().collect::<Vec<char>>();
	return parse_distinguished_name(&chars);
}

#[cfg(test)]
mod tests {

	#[test]
	fn test_empty() {
		let str = "";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 0);
	}

	#[test]
	fn single_attribute() {
		let str = "C=DE";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("C").unwrap() == "DE");
	}

	#[test]
	fn single_attribute_whitespace() {
		let str = "CN=Nyan Cat";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan Cat");
	}

	#[test]
	fn single_attribute_escaped_quote() {
		let str = "CN=Nyan \\\"Cat\\\"";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan \"Cat\"");
	}

	#[test]
	fn single_attribute_escaped_special() {
		let str = "CN=Nyan\\=Cat";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan=Cat");
	}

	#[test]
	fn single_attribute_escaped_escape() {
		let str = "CN=Nyan\\\\Cat";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan\\Cat");
	}

	#[test]
	fn single_attribute_hexpair() {
		let str = "CN=Nyan\\21Cat\\3F";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan!Cat?");
	}

	#[test]
	fn single_attribute_quoted() {
		let str = "C=\"DE\"";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("C").unwrap() == "DE");
	}

	#[test]
	fn single_attribute_quoted_whitespace() {
		let str = "CN=\"Nyan Cat\"";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan Cat");
	}

	#[test]
	fn single_attribute_quoted_escaped_quote() {
		let str = "CN=\"Nyan \\\"Cat\\\"\"";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan \"Cat\"");
	}

	#[test]
	fn single_attribute_quoted_escaped_special() {
		let str = "CN=\"Nyan\\=Cat\"";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan=Cat");
	}

	#[test]
	fn single_attribute_quoted_escaped_escape() {
		let str = "CN=\"Nyan\\\\Cat\"";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan\\Cat");
	}

	#[test]
	fn single_attribute_quoted_hexpair() {
		let str = "CN=\"Nyan\\21Cat\\3F\"";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "Nyan!Cat?");
	}

	#[test]
	fn single_attribute_hexstring() {
		let str = "CN=#4E59414E213F";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 1);
		assert!(res.attributes.get("CN").unwrap() == "NYAN!?");
	}

	#[test]
	fn invalid_quoting1() {
		let str = "CN=Nyan\"Cat";
		let res = ::parse_distinguished_name_str(str);
		assert!(res.is_err());
	}

	#[test]
	fn invalid_quoting2() {
		let str = "CN=\"NyanCat";
		let res = ::parse_distinguished_name_str(str);
		assert!(res.is_err());
	}

	#[test]
	fn invalid_hexstring() {
		let str = "CN=#abc";
		let res = ::parse_distinguished_name_str(str);
		assert!(res.is_err());
	}

	#[test]
	fn invalid_novalue() {
		let str = "CN";
		let res = ::parse_distinguished_name_str(str);
		assert!(res.is_err());
	}

	#[test]
	fn multiple_attributes() {
		let str = "C=DE,L=Berlin,ST=Berlin";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 3);
		assert!(res.attributes.get("C").unwrap() == "DE");
		assert!(res.attributes.get("L").unwrap() == "Berlin");
		assert!(res.attributes.get("ST").unwrap() == "Berlin");
	}

	#[test]
	fn multiple_attributes_alt() {
		let str = "C=DE,L=Berlin+ST=Berlin";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.len() == 3);
		assert!(res.attributes.get("C").unwrap() == "DE");
		assert!(res.attributes.get("L").unwrap() == "Berlin");
		assert!(res.attributes.get("ST").unwrap() == "Berlin");
	}

	#[test]
	fn full() {
		let str = "C=DE,CN=paul \\<test\\>,OU=ACME Inc.,O=ACME Inc.,L=Berlin,ST=Berlin";
		let res = ::parse_distinguished_name_str(str).unwrap();
		assert!(res.attributes.get("CN").unwrap() == "paul <test>");
		assert!(res.attributes.len() == 6);
		assert!(res.attributes.get("C").unwrap() == "DE");
		assert!(res.attributes.get("L").unwrap() == "Berlin");
		assert!(res.attributes.get("ST").unwrap() == "Berlin");
		assert!(res.attributes.get("O").unwrap() == "ACME Inc.");
		assert!(res.attributes.get("OU").unwrap() == "ACME Inc.");
	}

}


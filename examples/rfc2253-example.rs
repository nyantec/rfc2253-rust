extern crate rfc2253;

fn main() {
  let dn_str = "C=DE,CN=Hans Tester,OU=ACME Inc.,O=ACME Inc.,L=Berlin,ST=Berlin";
  let dn = rfc2253::parse_distinguished_name_str(dn_str).unwrap();

  println!("{:?}", dn);
  assert!(dn.attributes.len() == 6);
  assert!(dn.attributes.get("CN").unwrap() == "Hans Tester");
  assert!(dn.attributes.get("C").unwrap() == "DE");
  assert!(dn.attributes.get("L").unwrap() == "Berlin");
  assert!(dn.attributes.get("ST").unwrap() == "Berlin");
  assert!(dn.attributes.get("O").unwrap() == "ACME Inc.");
  assert!(dn.attributes.get("OU").unwrap() == "ACME Inc.");
}


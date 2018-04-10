rfc2253-rust
============

[![Build Status](https://travis-ci.org/nyantec/rfc2253-rust.svg?branch=master)](https://travis-ci.org/nyantec/rfc2253-rust)
![Crates.io](https://img.shields.io/crates/v/rfc2253.svg)

A small rust library that parses [RFC2253](https://www.ietf.org/rfc/rfc2253.txt)
encoded distinguished name strings. You can use this to decode the output of
openssl's `x509_NAME_print_ex(..., XN_FLAG_RFC2253)` or nginx's `$ssl_client_s_dn`
and `$ssl_client_i_dn` variables.


Usage
-----

Add this to your `Cargo.toml` file:

    [dependencies]
    rfc2253 = "*"

The example below shows how to parse a RFC2253 encoded distinguished name.

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


Build
-----

To build the `rfc2253-rust` library, run `cargo build`:

    $ cd rfc2253-rust
    $ cargo build

After you have made any changes to the code, run the test suite by executing
`cargo test`:

    $ cd rfc2253-rust
    $ cargo test


License
-------

    Copyright © 2018 Nyantec GmbH <oss@nyantec.com>

    Authors:
      Paul Asmuth <asm@nyantec.com>

    Provided that these terms and disclaimer and all copyright notices
    are retained or reproduced in an accompanying document, permission
    is granted to deal in this work without restriction, including un‐
    limited rights to use, publicly perform, distribute, sell, modify,
    merge, give away, or sublicence.

    This work is provided “AS IS” and WITHOUT WARRANTY of any kind, to
    the utmost extent permitted by applicable law, neither express nor
    implied; without malicious intent or gross negligence. In no event
    may a licensor, author or contributor be held liable for indirect,
    direct, other damage, loss, or other issues arising in any way out
    of dealing in the work, even if advised of the possibility of such
    damage or existence of a defect, except proven that it results out
    of said person’s immediate fault when using the work as intended.

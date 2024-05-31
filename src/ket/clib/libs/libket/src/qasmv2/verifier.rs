// SPDX-FileCopyrightText: 2024 Gabriel da Silva Cardoso <cardoso.gabriel@grad.ufsc.br>
//
// SPDX-License-Identifier: Apache-2.0

use openqasm as oq;
use oq::*;

pub fn verify_qasmv2(source: String) -> bool {
    let mut cache = SourceCache::new();
    let mut parser = Parser::new(&mut cache).with_file_policy(oq::parser::FilePolicy::Ignore);
    parser.parse_source::<String>(source, None);

    parser.done().to_errors().is_ok()
}

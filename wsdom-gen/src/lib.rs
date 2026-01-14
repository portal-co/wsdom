use std::{collections::BTreeMap, fmt::Display};

use itertools::Itertools;
use sha3::Digest;
pub fn gen<D: Display>(modules: &[D], rpcs: &BTreeMap<String, usize>) -> String {
    let modules = modules.iter().map(|a| format!("{a}")).collect_vec();
    const S: &str = include_str!("wsdom.ts");
    return format!(
        "{}\n{}",
        modules
            .iter()
            .enumerate()
            .map(|(i, m)| format!("import * as m{i} from '{m}'"))
            .join("\n"),
        S.replace(
            "$$x",
            &modules
                .iter()
                .enumerate()
                .map(|(i, m)| format!(
                    "_{} :m{i} as Value",
                    hex::encode(&sha3::Sha3_256::digest(m.as_bytes()))
                ))
                .join(",")
        )
        .replace(
            "$$e",
            &rpcs
                .iter()
                .map(|(a, v)| format!(
                    r#"public {a}({}): Promise<Value>{{
                        return new Promise((then) => {{
                            var i = 0;
                            while(this.#callbacks.has(i))i++;
                            this.#callbacks.set(i,then);
                            var s = `r{a}:${{i}};{};`;
                            (this.#sender)(s);
                        }});
                    }}"#,
                    (0usize..*v).map(|a| format!("param{a}: Value")).join(","),
                    (0usize..*v)
                        .map(|a| format!("${{this.#allocate(param{a})}}"))
                        .join(","),
                ))
                .join("\n")
        )
    );
}
pub fn launch(url: &str, path: &str, rpcs: &BTreeMap<String, usize>) -> String {
    return format!("import WSDOMConnectWebSocket from '{path}'\nexport const WS = WSDOMConnectToServer('{url}')\n{}",rpcs.iter().map(|(a,_)|format!("export const {a} = WS.{a};")).join(";"));
}

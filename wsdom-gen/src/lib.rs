use std::{collections::BTreeMap, fmt::Display};

use itertools::Itertools;
use sha3::Digest;
pub struct Module<D> {
    pub name: D,
    pub kind: ModuleKind,
}
pub enum ModuleKind {
    Injected,
    ESM,
}
pub fn gen<D: Display>(modules: &[Module<D>], rpcs: &BTreeMap<String, usize>) -> String {
    let modules2 = modules
        .iter()
        .map(|a| format!("{a}", a = &a.name))
        .collect_vec();
    const S: &str = include_str!("wsdom.ts");
    return format!(
        "{}\n{}",
        modules2
            .iter()
            .zip(modules.iter())
            .enumerate()
            .map(|(i, (m, Module { name, kind }))| match kind {
                ModuleKind::Injected => format!(""),
                ModuleKind::ESM => format!("import * as m{i} from '{m}'"),
            })
            .join("\n"),
        S.replace(
            "$$x",
            &modules2
                .iter()
                .zip(modules.iter())
                .enumerate()
                .map(|(i, (m, Module { name, kind }))| match kind {
                    ModuleKind::Injected => format!(
                        "get _{}(){{return self.#args.{name};}}",
                        hex::encode(&sha3::Sha3_256::digest(m.as_bytes()))
                    ),
                    ModuleKind::ESM => format!(
                        "_{} :m{i} as Value",
                        hex::encode(&sha3::Sha3_256::digest(m.as_bytes()))
                    ),
                })
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
        .replace(
            "$$a",
            &format!(
                "{{{}}}",
                modules
                    .iter()
                    .filter_map(|Module { name, kind }| match kind {
                        ModuleKind::Injected => Some(format!("{name}: unknown")),
                        ModuleKind::ESM => None,
                    })
                    .join(",")
            )
        )
    );
}
pub fn launch(url: &str, path: &str, rpcs: &BTreeMap<String, usize>) -> String {
    return format!("import WSDOMConnectWebSocket from '{path}'\nexport const WS = WSDOMConnectToServer('{url}')\n{}",rpcs.iter().map(|(a,_)|format!("export const {a} = WS.{a};")).join(";"));
}

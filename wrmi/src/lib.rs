pub use wrmi_macros::load_ts;

pub mod __wrmi_load_ts_macro {
    pub use ref_cast::RefCast;
    pub use wrmi_core::{js_types::*, Browser, JsCast, ToJs, UseInJsCode};
}

pub mod javascript {}

#[cfg(test)]
mod tests {
    #[test]
    fn history() {
        use super::__wrmi_load_ts_macro;
        wrmi_macros::load_ts!("../data-test/history.d.ts");
    }

    #[test]
    #[allow(non_snake_case, non_camel_case_types)]
    fn console() {
        use super::__wrmi_load_ts_macro;
        wrmi_macros::load_ts!("../data-test/console.d.ts");
    }

    #[test]
    fn math() {
        use super::__wrmi_load_ts_macro;
        wrmi_macros::load_ts!("../data-test/math.d.ts");
    }
}
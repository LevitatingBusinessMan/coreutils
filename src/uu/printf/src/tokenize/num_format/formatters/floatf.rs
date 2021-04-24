// spell-checker:ignore (ToDO) floatf inprefix

//! formatter for %f %F common-notation floating-point subs
use super::super::format_field::FormatField;
use super::super::formatter::{FormatPrimitive, Formatter, InPrefix};
use super::float_common::{get_primitive_dec, primitive_to_str_common, FloatAnalysis};

pub struct Floatf;
impl Floatf {
    pub fn new() -> Floatf {
        Floatf
    }
}
impl Formatter for Floatf {
    fn get_primitive(
        &self,
        field: &FormatField,
        inprefix: &InPrefix,
        str_in: &str,
    ) -> Option<FormatPrimitive> {
        let second_field = field.second_field.unwrap_or(6) + 1;
        let analysis =
            FloatAnalysis::analyze(&str_in, inprefix, None, Some(second_field as usize), false);
        let f = get_primitive_dec(
            inprefix,
            &str_in[inprefix.offset..],
            &analysis,
            second_field as usize,
            None,
        );
        Some(f)
    }
    fn primitive_to_str(&self, prim: &FormatPrimitive, field: FormatField) -> String {
        primitive_to_str_common(prim, &field)
    }
}

pub trait Params {
    fn num_params() -> usize;
    fn index_range() -> std::ops::Range<i32>;
    fn get_name(&self, index: i32) -> String;
    fn get_unit(&self, index: i32) -> String;
    fn is_button(&self, index: i32) -> bool;
    fn is_checkbox(&self, index: i32) -> bool;
    fn get_range(&self, index: i32) -> std::ops::RangeInclusive<f32>;
    fn get_value(&self, index: i32) -> f32;
    fn get_value_text(&self, index: i32) -> String;
    fn set_value(&self, index: i32, value: f32);
}

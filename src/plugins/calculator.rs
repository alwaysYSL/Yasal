use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::plugins::app_launcher::set_clipboard_text;
use crate::plugins::units::{self, Conversion};
use evalexpr::Value;
use std::sync::Arc;

pub struct CalculatorPlugin;

/// Decimals kept when showing the result of a calculation.
const MATH_DECIMALS: usize = 10;
/// Decimals kept when showing the result of a unit conversion.
const CONVERSION_DECIMALS: usize = 6;

impl CalculatorPlugin {
    pub fn new() -> Self {
        Self
    }

    fn looks_like_math(input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }
        let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
        let has_math_op = trimmed
            .chars()
            .any(|c| matches!(c, '+' | '-' | '*' | '/' | '^' | '%'));
        has_digit && has_math_op
    }

    /// Shared by `query` (unprefixed, must look like math) and `query_prefixed`
    /// (the user typed `=`, so anything evaluable is shown).
    fn answer(input: &str, explicit: bool) -> Vec<QueryResult> {
        let expression = input.trim_start_matches('=').trim();
        if expression.is_empty() {
            return Vec::new();
        }

        // "5 km to mi" has no operator, so conversions are tried before the math gate.
        if let Some(conversion) = units::convert_query(expression) {
            return vec![conversion_result(&conversion)];
        }

        if !explicit && !Self::looks_like_math(expression) {
            return Vec::new();
        }

        match evaluate(expression) {
            Some(value) => vec![math_result(expression, value)],
            None => Vec::new(),
        }
    }
}

/// Evaluates `expression` as real-number arithmetic.
///
/// evalexpr divides integers as integers (`10/4` is `2`, `1/3` is `0`), which is wrong
/// for a calculator, so integer literals are turned into floats first. Anything that is
/// not a finite number (a tuple from `1,5 + 2`, a division by zero, ...) yields `None`.
fn evaluate(expression: &str) -> Option<f64> {
    let prepared = floatify_integers(expression);
    let value = match evalexpr::eval(&prepared).ok()? {
        Value::Float(float) => float,
        Value::Int(int) => int as f64,
        _ => return None,
    };
    value.is_finite().then_some(value)
}

/// Rewrites integer literals as floats: `10/4` becomes `10.0/4.0`. Literals that are
/// already floats (`1.5`, `1e3`) and identifiers containing digits (`x1`) are kept.
fn floatify_integers(expression: &str) -> String {
    let chars: Vec<char> = expression.chars().collect();
    let mut out = String::with_capacity(expression.len() + 8);
    let mut i = 0;

    while i < chars.len() {
        let current = chars[i];
        let continues_word =
            i > 0 && (chars[i - 1].is_alphanumeric() || matches!(chars[i - 1], '_' | '.'));
        if !current.is_ascii_digit() || continues_word {
            out.push(current);
            i += 1;
            continue;
        }

        // A numeric literal starts here: integer part, optional fraction, optional exponent.
        while i < chars.len() && chars[i].is_ascii_digit() {
            out.push(chars[i]);
            i += 1;
        }
        let mut is_float = false;

        if chars.get(i) == Some(&'.') {
            is_float = true;
            out.push('.');
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                out.push(chars[i]);
                i += 1;
            }
        }

        if matches!(chars.get(i), Some('e' | 'E')) {
            let has_sign = matches!(chars.get(i + 1), Some('+' | '-'));
            let digit_index = i + 1 + usize::from(has_sign);
            if chars.get(digit_index).is_some_and(|c| c.is_ascii_digit()) {
                is_float = true;
                out.push(chars[i]);
                i += 1;
                if has_sign {
                    out.push(chars[i]);
                    i += 1;
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    out.push(chars[i]);
                    i += 1;
                }
            }
        }

        let followed_by_word = chars.get(i).is_some_and(|c| c.is_alphabetic() || *c == '_');
        if !is_float && !followed_by_word {
            out.push_str(".0");
        }
    }

    out
}

fn copy_action(text: String) -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(move || {
        let _ = set_clipboard_text(&text);
    })
}

fn math_result(expression: &str, value: f64) -> QueryResult {
    let result = units::format_number(value, MATH_DECIMALS);
    let copy = copy_action(result.clone());

    QueryResult {
        id: format!("calc:{result}"),
        title: format!("= {result}"),
        subtitle: Some(format!("Ekspresi: {expression} (Tekan Enter untuk salin)")),
        category: "Kalkulator".to_string(),
        icon: ItemIcon::Symbolic("calc"),
        score: 950, // High priority when math is valid
        primary_action: Arc::clone(&copy),
        secondary_action: Some(ItemAction {
            label: "Salin Hasil".to_string(),
            icon: "📋",
            shortcut_hint: Some("Enter / Ctrl+Enter".to_string()),
            action: copy,
        }),
        additional_actions: Vec::new(),
    }
}

fn conversion_result(conversion: &Conversion) -> QueryResult {
    let result = units::format_number(conversion.value, CONVERSION_DECIMALS);
    let input = units::format_number(conversion.input_value, CONVERSION_DECIMALS);
    let with_unit = format!("{result} {}", conversion.to_symbol);

    QueryResult {
        id: format!("calc:unit:{with_unit}"),
        title: format!("= {with_unit}"),
        subtitle: Some(format!(
            "{input} {} = {with_unit} · Konversi satuan (Enter untuk salin angka)",
            conversion.from_symbol
        )),
        category: "Kalkulator".to_string(),
        icon: ItemIcon::Symbolic("calc"),
        score: 950,
        // Enter copies the bare number so it can be pasted into a calculation or form;
        // Ctrl+Enter keeps the unit.
        primary_action: copy_action(result),
        secondary_action: Some(ItemAction {
            label: "Salin dengan satuan".to_string(),
            icon: "📋",
            shortcut_hint: Some("Ctrl+Enter".to_string()),
            action: copy_action(with_unit),
        }),
        additional_actions: Vec::new(),
    }
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CalculatorPlugin {
    fn id(&self) -> &str {
        "calculator"
    }

    fn name(&self) -> &str {
        "Calculator"
    }

    fn prefix(&self) -> Option<&str> {
        Some("=")
    }

    fn is_global(&self) -> bool {
        true
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        Self::answer(input, input.starts_with('='))
    }

    fn query_prefixed(&self, input: &str) -> Vec<QueryResult> {
        // The dispatcher already removed the `=`; the user asked for a calculation, so do
        // not require an operator ("= math::sqrt(16)", "= 5").
        Self::answer(input, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn title(input: &str) -> Option<String> {
        CalculatorPlugin::new()
            .query(input)
            .first()
            .map(|result| result.title.clone())
    }

    #[test]
    fn integer_division_is_real_division() {
        assert_eq!(title("10/4").as_deref(), Some("= 2.5"));
        assert_eq!(title("7/2").as_deref(), Some("= 3.5"));
        assert_eq!(title("1/3").as_deref(), Some("= 0.3333333333"));
        assert_eq!(title("100 / 4").as_deref(), Some("= 25"));
    }

    #[test]
    fn floating_point_noise_is_hidden() {
        assert_eq!(title("0.1+0.2").as_deref(), Some("= 0.3"));
        assert_eq!(title("100*1.1").as_deref(), Some("= 110"));
        assert_eq!(title("2^3").as_deref(), Some("= 8"));
        assert_eq!(title("5 % 3").as_deref(), Some("= 2"));
        assert_eq!(title("(1+2)*3").as_deref(), Some("= 9"));
        assert_eq!(title("1e3+1").as_deref(), Some("= 1001"));
        assert_eq!(title("1e-2*5").as_deref(), Some("= 0.05"));
    }

    #[test]
    fn invalid_or_non_numeric_results_are_dropped() {
        assert_eq!(title("10 / 0"), None);
        assert_eq!(title("1,5 + 2"), None);
        assert_eq!(title("hello world"), None);
        assert_eq!(title(""), None);
    }

    #[test]
    fn an_explicit_prefix_does_not_need_an_operator() {
        let calc = CalculatorPlugin::new();
        // The dispatcher strips the "=" before calling query_prefixed.
        assert_eq!(calc.query_prefixed("math::sqrt(16)")[0].title, "= 4");
        assert_eq!(calc.query_prefixed("5")[0].title, "= 5");
        // Without the prefix, a bare number or function call is left to other plugins.
        assert!(calc.query("5").is_empty());
        assert!(calc.query("math::sqrt(16)").is_empty());
    }

    #[test]
    fn unit_conversions_work_with_and_without_the_prefix() {
        let calc = CalculatorPlugin::new();
        assert_eq!(calc.query("5 km to mi")[0].title, "= 3.106856 mi");
        assert_eq!(calc.query("= 100 c to f")[0].title, "= 212 °F");
        assert_eq!(calc.query_prefixed("1.5 jam ke menit")[0].title, "= 90 min");
        assert!(calc.query("5 km to kg").is_empty());
    }

    #[test]
    fn conversion_results_offer_a_copy_with_unit_action() {
        let result = &CalculatorPlugin::new().query("5 km to m")[0];
        assert_eq!(result.category, "Kalkulator");
        assert_eq!(
            result.secondary_action.as_ref().map(|a| a.label.as_str()),
            Some("Salin dengan satuan")
        );
    }

    #[test]
    fn floatify_keeps_existing_floats_and_identifiers() {
        assert_eq!(floatify_integers("10/4"), "10.0/4.0");
        assert_eq!(floatify_integers("1.5+2"), "1.5+2.0");
        assert_eq!(floatify_integers("1e3+1"), "1e3+1.0");
        assert_eq!(floatify_integers("1e+3"), "1e+3");
        assert_eq!(floatify_integers("x1+2"), "x1+2.0");
        assert_eq!(floatify_integers("math::sqrt(16)"), "math::sqrt(16.0)");
    }
}

//! Offline unit conversion and number formatting for the calculator.
//!
//! Queries look like `5 km to mi`, `72 f in c`, `1.5 jam ke menit` or `100kg to lb`.
//! Units are matched case-insensitively by symbol or by an English/Indonesian name.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Length,
    Mass,
    Volume,
    Temperature,
    Time,
    Data,
    Area,
    Speed,
}

struct Unit {
    category: Category,
    /// Symbol shown in results.
    symbol: &'static str,
    /// Lower-case names accepted in queries (the symbol itself is always accepted).
    names: &'static [&'static str],
    /// Multiplier to the category's base unit. Unused for temperature.
    factor: f64,
}

const fn unit(
    category: Category,
    symbol: &'static str,
    names: &'static [&'static str],
    factor: f64,
) -> Unit {
    Unit {
        category,
        symbol,
        names,
        factor,
    }
}

use Category::*;

// Base units: metre, kilogram, litre, second, byte, square metre, metre/second.
// Data uses SI prefixes (1 KB = 1000 B) like most web tools; KiB/MiB/... are binary.
static UNITS: &[Unit] = &[
    // Length
    unit(
        Length,
        "mm",
        &["milimeter", "millimeter", "millimeters"],
        0.001,
    ),
    unit(
        Length,
        "cm",
        &["sentimeter", "centimeter", "centimeters"],
        0.01,
    ),
    unit(Length, "m", &["meter", "meters", "metre", "metres"], 1.0),
    unit(Length, "km", &["kilometer", "kilometers"], 1000.0),
    unit(Length, "in", &["inch", "inches", "inci"], 0.0254),
    unit(Length, "ft", &["foot", "feet", "kaki"], 0.3048),
    unit(Length, "yd", &["yard", "yards"], 0.9144),
    unit(Length, "mi", &["mile", "miles"], 1609.344),
    // Mass
    unit(Mass, "mg", &["miligram", "milligram"], 0.000_001),
    unit(Mass, "g", &["gram", "grams"], 0.001),
    unit(Mass, "kg", &["kilogram", "kilograms", "kilo"], 1.0),
    unit(Mass, "ons", &[], 0.1),
    unit(Mass, "t", &["ton", "tons", "tonne", "tonnes"], 1000.0),
    unit(Mass, "lb", &["lbs", "pound", "pounds"], 0.453_592_37),
    unit(Mass, "oz", &["ounce", "ounces"], 0.028_349_523_125),
    // Volume
    unit(Volume, "ml", &["mililiter", "milliliter"], 0.001),
    unit(Volume, "l", &["liter", "liters", "litre", "litres"], 1.0),
    unit(Volume, "m³", &["m3"], 1000.0),
    unit(
        Volume,
        "gal",
        &["gallon", "gallons", "galon"],
        3.785_411_784,
    ),
    unit(Volume, "cup", &["cups"], 0.236_588_236_5),
    unit(
        Volume,
        "tbsp",
        &["tablespoon", "tablespoons"],
        0.014_786_764_781_25,
    ),
    unit(
        Volume,
        "tsp",
        &["teaspoon", "teaspoons"],
        0.004_928_921_593_75,
    ),
    // Temperature (factor unused)
    unit(Temperature, "°C", &["c", "celsius"], 1.0),
    unit(Temperature, "°F", &["f", "fahrenheit"], 1.0),
    unit(Temperature, "K", &["kelvin"], 1.0),
    // Time
    unit(
        Time,
        "ms",
        &["milidetik", "millisecond", "milliseconds"],
        0.001,
    ),
    unit(Time, "s", &["sec", "detik", "second", "seconds"], 1.0),
    unit(Time, "min", &["menit", "minute", "minutes"], 60.0),
    unit(Time, "jam", &["h", "hr", "hour", "hours"], 3600.0),
    unit(Time, "hari", &["d", "day", "days"], 86_400.0),
    unit(Time, "minggu", &["wk", "week", "weeks"], 604_800.0),
    // Data
    unit(Data, "bit", &["bits"], 0.125),
    unit(Data, "B", &["byte", "bytes"], 1.0),
    unit(Data, "KB", &["kilobyte", "kilobytes"], 1e3),
    unit(Data, "MB", &["megabyte", "megabytes"], 1e6),
    unit(Data, "GB", &["gigabyte", "gigabytes"], 1e9),
    unit(Data, "TB", &["terabyte", "terabytes"], 1e12),
    unit(Data, "KiB", &["kibibyte"], 1024.0),
    unit(Data, "MiB", &["mebibyte"], 1_048_576.0),
    unit(Data, "GiB", &["gibibyte"], 1_073_741_824.0),
    unit(Data, "TiB", &["tebibyte"], 1_099_511_627_776.0),
    // Area
    unit(Area, "m²", &["m2", "sqm"], 1.0),
    unit(Area, "cm²", &["cm2"], 0.0001),
    unit(Area, "km²", &["km2"], 1_000_000.0),
    unit(Area, "ha", &["hektar", "hectare", "hectares"], 10_000.0),
    unit(Area, "acre", &["acres"], 4_046.856_422_4),
    unit(Area, "ft²", &["ft2", "sqft"], 0.092_903_04),
    // Speed
    unit(Speed, "m/s", &["mps"], 1.0),
    unit(Speed, "km/h", &["kmh", "kph", "km/j", "kmj"], 1.0 / 3.6),
    unit(Speed, "mph", &["mi/h"], 0.447_04),
    unit(Speed, "knot", &["knots", "kn", "kt"], 1852.0 / 3600.0),
];

fn find_unit(token: &str) -> Option<&'static Unit> {
    let token = token.to_lowercase();
    UNITS.iter().find(|unit| {
        unit.symbol.to_lowercase() == token || unit.names.iter().any(|name| *name == token)
    })
}

/// Parses a decimal number. `.` is the decimal separator; a lone `,` followed by other
/// than three digits is accepted as a decimal comma (`1,5`), and `1,000` is read as a
/// thousands separator so the common "1,000 m" means one thousand.
pub fn parse_number(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    let normalized = if text.contains(',') && !text.contains('.') {
        let mut parts = text.split(',');
        let head = parts.next().unwrap_or_default();
        let tail: Vec<&str> = parts.collect();
        if tail.len() == 1 && tail[0].len() != 3 {
            format!("{head}.{}", tail[0])
        } else {
            text.replace(',', "")
        }
    } else {
        text.replace(',', "")
    };

    // Reject forms f64::from_str accepts that are not plain numbers ("inf", "nan", "1e5x").
    let numeric = normalized
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'));
    if !numeric {
        return None;
    }
    normalized
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn to_celsius(value: f64, unit: &Unit) -> f64 {
    match unit.symbol {
        "°F" => (value - 32.0) * 5.0 / 9.0,
        "K" => value - 273.15,
        _ => value,
    }
}

fn from_celsius(celsius: f64, unit: &Unit) -> f64 {
    match unit.symbol {
        "°F" => celsius * 9.0 / 5.0 + 32.0,
        "K" => celsius + 273.15,
        _ => celsius,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Conversion {
    pub input_value: f64,
    pub from_symbol: &'static str,
    pub value: f64,
    pub to_symbol: &'static str,
    pub category: Category,
}

fn is_separator(token: &str) -> bool {
    matches!(token, "to" | "in" | "ke" | "->" | "=>")
}

/// Splits `5km` into (`5`, `km`). Returns `None` when there is no leading number.
fn split_number_and_unit(token: &str) -> Option<(&str, &str)> {
    let boundary = token
        .char_indices()
        .find(|(_, c)| c.is_alphabetic() || matches!(c, '°' | '²' | '³'))
        .map(|(index, _)| index)?;
    (boundary > 0).then(|| token.split_at(boundary))
}

/// Parses and performs a conversion such as `5 km to mi`. `None` if the text is not a
/// conversion, a unit is unknown, or the units measure different things.
pub fn convert_query(input: &str) -> Option<Conversion> {
    let lowered = input.trim().to_lowercase();
    let tokens: Vec<&str> = lowered.split_whitespace().collect();

    let (number_text, from_text, to_text) = match tokens.as_slice() {
        [number, from, separator, to] if is_separator(separator) => (*number, *from, *to),
        [joined, separator, to] if is_separator(separator) => {
            let (number, from) = split_number_and_unit(joined)?;
            (number, from, *to)
        }
        _ => return None,
    };

    let input_value = parse_number(number_text)?;
    let from = find_unit(from_text)?;
    let to = find_unit(to_text)?;
    if from.category != to.category {
        return None;
    }

    let value = if from.category == Temperature {
        from_celsius(to_celsius(input_value, from), to)
    } else {
        input_value * from.factor / to.factor
    };
    if !value.is_finite() {
        return None;
    }

    Some(Conversion {
        input_value,
        from_symbol: from.symbol,
        value,
        to_symbol: to.symbol,
        category: from.category,
    })
}

/// Formats a number without floating point noise: `0.1 + 0.2` becomes `0.3`,
/// whole numbers lose their `.0`, and very large or tiny values use scientific notation.
pub fn format_number(value: f64, max_decimals: usize) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    let magnitude = value.abs();
    let smallest_shown = 10f64.powi(-(max_decimals as i32));
    if magnitude != 0.0 && (magnitude >= 1e15 || magnitude < smallest_shown) {
        return format!("{value:e}");
    }

    let mut text = format!("{value:.max_decimals$}");
    if text.contains('.') {
        text = text.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if text == "-0" {
        text = "0".to_string();
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convert(input: &str) -> Option<String> {
        convert_query(input).map(|c| format!("{} {}", format_number(c.value, 6), c.to_symbol))
    }

    #[test]
    fn converts_common_units_in_several_phrasings() {
        assert_eq!(convert("5 km to mi").as_deref(), Some("3.106856 mi"));
        assert_eq!(convert("5km to m").as_deref(), Some("5000 m"));
        assert_eq!(convert("1 mi in km").as_deref(), Some("1.609344 km"));
        assert_eq!(convert("100 KG TO LB").as_deref(), Some("220.462262 lb"));
        assert_eq!(convert("1.5 jam ke menit").as_deref(), Some("90 min"));
        assert_eq!(convert("2 gb to mb").as_deref(), Some("2000 MB"));
        assert_eq!(convert("1 gib to mib").as_deref(), Some("1024 MiB"));
        assert_eq!(convert("1 ha -> m2").as_deref(), Some("10000 m²"));
        assert_eq!(convert("72 km/h to m/s").as_deref(), Some("20 m/s"));
    }

    #[test]
    fn the_inch_unit_does_not_clash_with_the_in_keyword() {
        assert_eq!(convert("1 in in cm").as_deref(), Some("2.54 cm"));
        assert_eq!(convert("10 cm to in").as_deref(), Some("3.937008 in"));
    }

    #[test]
    fn converts_temperatures_including_negative_values() {
        assert_eq!(convert("100 c to f").as_deref(), Some("212 °F"));
        assert_eq!(convert("32 f to c").as_deref(), Some("0 °C"));
        assert_eq!(convert("0 c to k").as_deref(), Some("273.15 K"));
        assert_eq!(convert("-40 c to f").as_deref(), Some("-40 °F"));
    }

    #[test]
    fn rejects_unknown_incompatible_and_non_conversion_input() {
        assert_eq!(convert("5 km to kg"), None); // length vs mass
        assert_eq!(convert("5 foo to m"), None);
        assert_eq!(convert("km to mi"), None); // no number
        assert_eq!(convert("word to pdf"), None);
        assert_eq!(convert("5 km"), None);
        assert_eq!(convert("1 + 2"), None);
        assert_eq!(convert(""), None);
    }

    #[test]
    fn parses_decimal_comma_and_thousands_separator() {
        assert_eq!(parse_number("1.5"), Some(1.5));
        assert_eq!(parse_number("1,5"), Some(1.5));
        assert_eq!(parse_number("1,000"), Some(1000.0));
        assert_eq!(parse_number("-2.5"), Some(-2.5));
        assert_eq!(parse_number("1e3"), Some(1000.0));
        assert_eq!(parse_number("inf"), None);
        assert_eq!(parse_number("abc"), None);
    }

    #[test]
    fn formats_numbers_without_floating_point_noise() {
        assert_eq!(format_number(0.1 + 0.2, 10), "0.3");
        assert_eq!(format_number(100.0 * 1.1, 10), "110");
        assert_eq!(format_number(8.0, 10), "8");
        assert_eq!(format_number(-0.0, 10), "0");
        assert_eq!(format_number(1.0 / 3.0, 6), "0.333333");
        assert_eq!(format_number(1e-12, 10), "1e-12");
        assert_eq!(format_number(2e15, 10), "2e15");
        assert_eq!(format_number(-12.5, 10), "-12.5");
    }
}

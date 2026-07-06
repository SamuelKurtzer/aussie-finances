pub fn fmt_int_commas(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let grouped: String = out.chars().rev().collect();
    format!("{sign}{grouped}")
}

pub fn fmt_money(value: f64) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    let whole = abs.trunc() as i64;
    let cents = ((abs - whole as f64) * 100.0).round() as i64;
    format!("{sign}${}.{:02}", fmt_int_commas(whole), cents)
}

pub fn fmt_currency(value: f64) -> String {
    format!("${}", fmt_int_commas(value.round() as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_int_commas_zero() {
        assert_eq!(fmt_int_commas(0), "0");
    }

    #[test]
    fn fmt_int_commas_negative() {
        assert_eq!(fmt_int_commas(-1234), "-1,234");
    }

    #[test]
    fn fmt_int_commas_large() {
        assert_eq!(fmt_int_commas(1_234_567_890), "1,234,567,890");
    }

    #[test]
    fn fmt_int_commas_small() {
        assert_eq!(fmt_int_commas(42), "42");
    }

    #[test]
    fn fmt_money_zero() {
        assert_eq!(fmt_money(0.0), "$0.00");
    }

    #[test]
    fn fmt_money_negative() {
        assert_eq!(fmt_money(-1234.56), "-$1,234.56");
    }

    #[test]
    fn fmt_money_large() {
        assert_eq!(fmt_money(1_000_000.99), "$1,000,000.99");
    }

    #[test]
    fn fmt_money_fractional_cent_rounds() {
        assert_eq!(fmt_money(1234.567), "$1,234.57");
    }

    #[test]
    fn fmt_currency_zero() {
        assert_eq!(fmt_currency(0.0), "$0");
    }

    #[test]
    fn fmt_currency_rounds() {
        assert_eq!(fmt_currency(1234.56), "$1,235");
    }

    #[test]
    fn fmt_currency_negative() {
        assert_eq!(fmt_currency(-999.4), "$-999");
    }
}

pub mod chart_colors {
    pub const BALANCE: &str = "#e5c07b";
    pub const INTEREST: &str = "#e06c75";
    pub const OFFSET: &str = "#61afef";
    pub const REPAID: &str = "#98c379";
    pub const DR_OFFSET: &str = "#61afef";
    pub const DR_INVESTMENT: &str = "#98c379";
    pub const DR_RECYCLED_DEBT: &str = "#e06c75";
    pub const DR_DEDUCTIBLE: &str = "#c678dd";
}

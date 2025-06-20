//! SI prefix definitions and utilities

/// SI prefixes for units
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prefix {
    // Large prefixes (10^n where n > 0)
    Yotta, // 10^24
    Zetta, // 10^21
    Exa,   // 10^18
    Peta,  // 10^15
    Tera,  // 10^12
    Giga,  // 10^9
    Mega,  // 10^6
    Kilo,  // 10^3
    Hecto, // 10^2
    Deca,  // 10^1

    // Small prefixes (10^n where n < 0)
    Deci,  // 10^-1
    Centi, // 10^-2
    Milli, // 10^-3
    Micro, // 10^-6
    Nano,  // 10^-9
    Pico,  // 10^-12
    Femto, // 10^-15
    Atto,  // 10^-18
    Zepto, // 10^-21
    Yocto, // 10^-24
}

/// Binary prefixes for data units (powers of 1024)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryPrefix {
    Ki, // 2^10 = 1024
    Mi, // 2^20 = 1024^2
    Gi, // 2^30 = 1024^3
    Ti, // 2^40 = 1024^4
    Pi, // 2^50 = 1024^5
    Ei, // 2^60 = 1024^6
}

impl BinaryPrefix {
    /// Get the multiplication factor for this binary prefix
    pub fn factor(&self) -> f64 {
        match self {
            BinaryPrefix::Ki => 1024.0,
            BinaryPrefix::Mi => 1_048_576.0,
            BinaryPrefix::Gi => 1_073_741_824.0,
            BinaryPrefix::Ti => 1_099_511_627_776.0,
            BinaryPrefix::Pi => 1_125_899_906_842_624.0,
            BinaryPrefix::Ei => 1_152_921_504_606_846_976.0,
        }
    }

    /// Get the symbol for this binary prefix
    pub fn symbol(&self) -> &'static str {
        match self {
            BinaryPrefix::Ki => "Ki",
            BinaryPrefix::Mi => "Mi",
            BinaryPrefix::Gi => "Gi",
            BinaryPrefix::Ti => "Ti",
            BinaryPrefix::Pi => "Pi",
            BinaryPrefix::Ei => "Ei",
        }
    }

    /// Get the full name of this binary prefix
    pub fn name(&self) -> &'static str {
        match self {
            BinaryPrefix::Ki => "kibi",
            BinaryPrefix::Mi => "mebi",
            BinaryPrefix::Gi => "gibi",
            BinaryPrefix::Ti => "tebi",
            BinaryPrefix::Pi => "pebi",
            BinaryPrefix::Ei => "exbi",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeWords {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
    Decade,
    Century,
    Millenium,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Time {
    TimeSeconds(Prefix, f64),
    TimeWords(TimeWords, f64),
}
#[derive(Debug, Clone, PartialEq)]
pub enum BaseData {
    Bits,
    Bytes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    prefix: Prefix,
    base: BaseData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rate {
    base: Box<Unit>,
    per_time: Time,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Time(Time),
    Data(Data),
    // Could be something like "requests" or "$"
    Arbitrary(String),
    Ratio(f64),
    Rate(Rate),
}

impl Unit {
    fn allows_addition(a: &Unit, b: &Unit) -> bool {
        match (a, b) {
            (Unit::Time(_), Unit::Time(_)) => true,
            (Unit::Data(_), Unit::Data(_)) => true,
            (Unit::Ratio(_), Unit::Ratio(_)) => true,
            (Unit::Arbitrary(a_str), Unit::Arbitrary(b_str)) => a_str == b_str,
            (Unit::Rate(Rate { base: a_base, .. }), Unit::Rate(Rate { base: b_base, .. })) => {
                a_base == b_base
            }
            (Unit::Time(time), Unit::Data(data)) => todo!(),
            (Unit::Time(time), Unit::Arbitrary(_)) => todo!(),
            (Unit::Time(time), Unit::Ratio(_)) => todo!(),
            (Unit::Time(time), Unit::Rate(rate)) => todo!(),
            (Unit::Data(data), Unit::Time(time)) => todo!(),
            (Unit::Data(data), Unit::Arbitrary(_)) => todo!(),
            (Unit::Data(data), Unit::Ratio(_)) => todo!(),
            (Unit::Data(data), Unit::Rate(rate)) => todo!(),
            (Unit::Arbitrary(_), Unit::Time(time)) => todo!(),
            (Unit::Arbitrary(_), Unit::Data(data)) => todo!(),
            (Unit::Arbitrary(_), Unit::Ratio(_)) => todo!(),
            (Unit::Arbitrary(_), Unit::Rate(rate)) => todo!(),
            (Unit::Ratio(_), Unit::Time(time)) => todo!(),
            (Unit::Ratio(_), Unit::Data(data)) => todo!(),
            (Unit::Ratio(_), Unit::Arbitrary(_)) => todo!(),
            (Unit::Ratio(_), Unit::Rate(rate)) => todo!(),
            (Unit::Rate(rate), Unit::Time(time)) => todo!(),
            (Unit::Rate(rate), Unit::Data(data)) => todo!(),
            (Unit::Rate(rate), Unit::Arbitrary(_)) => todo!(),
            (Unit::Rate(rate), Unit::Ratio(_)) => todo!(),
        }
    }
}

impl Prefix {
    /// Get the multiplication factor for this prefix
    pub fn factor(&self) -> f64 {
        match self {
            // Large prefixes
            Prefix::Yotta => 1e24,
            Prefix::Zetta => 1e21,
            Prefix::Exa => 1e18,
            Prefix::Peta => 1e15,
            Prefix::Tera => 1e12,
            Prefix::Giga => 1e9,
            Prefix::Mega => 1e6,
            Prefix::Kilo => 1e3,
            Prefix::Hecto => 1e2,
            Prefix::Deca => 1e1,

            // Small prefixes
            Prefix::Deci => 1e-1,
            Prefix::Centi => 1e-2,
            Prefix::Milli => 1e-3,
            Prefix::Micro => 1e-6,
            Prefix::Nano => 1e-9,
            Prefix::Pico => 1e-12,
            Prefix::Femto => 1e-15,
            Prefix::Atto => 1e-18,
            Prefix::Zepto => 1e-21,
            Prefix::Yocto => 1e-24,
        }
    }

    /// Get the symbol for this prefix
    pub fn symbol(&self) -> &'static str {
        match self {
            // Large prefixes
            Prefix::Yotta => "Y",
            Prefix::Zetta => "Z",
            Prefix::Exa => "E",
            Prefix::Peta => "P",
            Prefix::Tera => "T",
            Prefix::Giga => "G",
            Prefix::Mega => "M",
            Prefix::Kilo => "k",
            Prefix::Hecto => "h",
            Prefix::Deca => "da",

            // Small prefixes
            Prefix::Deci => "d",
            Prefix::Centi => "c",
            Prefix::Milli => "m",
            Prefix::Micro => "μ",
            Prefix::Nano => "n",
            Prefix::Pico => "p",
            Prefix::Femto => "f",
            Prefix::Atto => "a",
            Prefix::Zepto => "z",
            Prefix::Yocto => "y",
        }
    }

    /// Get the full name of this prefix
    pub fn name(&self) -> &'static str {
        match self {
            // Large prefixes
            Prefix::Yotta => "yotta",
            Prefix::Zetta => "zetta",
            Prefix::Exa => "exa",
            Prefix::Peta => "peta",
            Prefix::Tera => "tera",
            Prefix::Giga => "giga",
            Prefix::Mega => "mega",
            Prefix::Kilo => "kilo",
            Prefix::Hecto => "hecto",
            Prefix::Deca => "deca",

            // Small prefixes
            Prefix::Deci => "deci",
            Prefix::Centi => "centi",
            Prefix::Milli => "milli",
            Prefix::Micro => "micro",
            Prefix::Nano => "nano",
            Prefix::Pico => "pico",
            Prefix::Femto => "femto",
            Prefix::Atto => "atto",
            Prefix::Zepto => "zepto",
            Prefix::Yocto => "yocto",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_factors() {
        assert_eq!(Prefix::Kilo.factor(), 1e3);
        assert_eq!(Prefix::Mega.factor(), 1e6);
        assert_eq!(Prefix::Giga.factor(), 1e9);
        assert_eq!(Prefix::Tera.factor(), 1e12);
        assert_eq!(Prefix::Peta.factor(), 1e15);
        assert_eq!(Prefix::Exa.factor(), 1e18);
        assert_eq!(Prefix::Zetta.factor(), 1e21);
        assert_eq!(Prefix::Yotta.factor(), 1e24);

        assert_eq!(Prefix::Milli.factor(), 1e-3);
        assert_eq!(Prefix::Micro.factor(), 1e-6);
        assert_eq!(Prefix::Nano.factor(), 1e-9);
        assert_eq!(Prefix::Pico.factor(), 1e-12);
        assert_eq!(Prefix::Femto.factor(), 1e-15);
        assert_eq!(Prefix::Atto.factor(), 1e-18);
        assert_eq!(Prefix::Zepto.factor(), 1e-21);
        assert_eq!(Prefix::Yocto.factor(), 1e-24);

        assert_eq!(Prefix::Hecto.factor(), 1e2);
        assert_eq!(Prefix::Deca.factor(), 1e1);
        assert_eq!(Prefix::Deci.factor(), 1e-1);
        assert_eq!(Prefix::Centi.factor(), 1e-2);
    }

    #[test]
    fn test_prefix_symbols() {
        assert_eq!(Prefix::Kilo.symbol(), "k");
        assert_eq!(Prefix::Mega.symbol(), "M");
        assert_eq!(Prefix::Giga.symbol(), "G");
        assert_eq!(Prefix::Milli.symbol(), "m");
        assert_eq!(Prefix::Micro.symbol(), "μ");
        assert_eq!(Prefix::Nano.symbol(), "n");
    }

    #[test]
    fn test_prefix_names() {
        assert_eq!(Prefix::Kilo.name(), "kilo");
        assert_eq!(Prefix::Mega.name(), "mega");
        assert_eq!(Prefix::Giga.name(), "giga");
        assert_eq!(Prefix::Milli.name(), "milli");
        assert_eq!(Prefix::Micro.name(), "micro");
        assert_eq!(Prefix::Nano.name(), "nano");
    }

    #[test]
    fn test_binary_prefix_factors() {
        assert_eq!(BinaryPrefix::Ki.factor(), 1024.0);
        assert_eq!(BinaryPrefix::Mi.factor(), 1_048_576.0);
        assert_eq!(BinaryPrefix::Gi.factor(), 1_073_741_824.0);
        assert_eq!(BinaryPrefix::Ti.factor(), 1_099_511_627_776.0);
        assert_eq!(BinaryPrefix::Pi.factor(), 1_125_899_906_842_624.0);
        assert_eq!(BinaryPrefix::Ei.factor(), 1_152_921_504_606_846_976.0);
    }

    #[test]
    fn test_binary_prefix_symbols() {
        assert_eq!(BinaryPrefix::Ki.symbol(), "Ki");
        assert_eq!(BinaryPrefix::Mi.symbol(), "Mi");
        assert_eq!(BinaryPrefix::Gi.symbol(), "Gi");
        assert_eq!(BinaryPrefix::Ti.symbol(), "Ti");
        assert_eq!(BinaryPrefix::Pi.symbol(), "Pi");
        assert_eq!(BinaryPrefix::Ei.symbol(), "Ei");
    }

    #[test]
    fn test_binary_prefix_names() {
        assert_eq!(BinaryPrefix::Ki.name(), "kibi");
        assert_eq!(BinaryPrefix::Mi.name(), "mebi");
        assert_eq!(BinaryPrefix::Gi.name(), "gibi");
        assert_eq!(BinaryPrefix::Ti.name(), "tebi");
        assert_eq!(BinaryPrefix::Pi.name(), "pebi");
        assert_eq!(BinaryPrefix::Ei.name(), "exbi");
    }
}

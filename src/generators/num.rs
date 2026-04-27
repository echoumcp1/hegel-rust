use super::{BasicGenerator, Generator, TestCase};
use crate::utils::cbor_utils::{cbor_map, map_get};
use crate::utils::num::{cbor_to_bigint, cbor_to_biguint, int_to_cbor};
use ciborium::Value;
use num_bigint::{BigInt, BigUint};
use num_complex::Complex;
use num_integer::Integer as NumInteger;
use num_rational::Ratio;
use num_traits::{CheckedMul, Num, One, Zero};

// ---------------------------------------------------------------------------
// Integer impls for BigInt/BigUint
// ---------------------------------------------------------------------------

impl super::Integer for BigInt {
    fn default_min() -> Self {
        -(<BigInt as One>::one() << 128u32)
    }
    fn default_max() -> Self {
        (<BigInt as One>::one() << 128u32) - <BigInt as One>::one()
    }
    fn one() -> Self {
        <BigInt as One>::one()
    }
    fn to_cbor(&self) -> Value {
        int_to_cbor(self.clone())
    }
    fn from_cbor(v: Value) -> Self {
        cbor_to_bigint(v)
    }
}

impl super::Integer for BigUint {
    fn default_min() -> Self {
        BigUint::zero()
    }
    fn default_max() -> Self {
        (<BigUint as One>::one() << 128u32) - <BigUint as One>::one()
    }
    fn one() -> Self {
        <BigUint as One>::one()
    }
    fn to_cbor(&self) -> Value {
        int_to_cbor(BigInt::from(self.clone()))
    }
    fn from_cbor(v: Value) -> Self {
        cbor_to_biguint(v)
    }
}

// ---------------------------------------------------------------------------
// RationalGenerator
// ---------------------------------------------------------------------------

/// Generator for [`Ratio<T>`] values. Created by [`rationals()`].
///
/// Generates a numerator and denominator independently, with the denominator
/// constrained to be non-zero. The resulting `Ratio` is automatically reduced
/// to lowest terms by `Ratio::new()`.
pub struct RationalGenerator<T> {
    min: Option<T>,
    max: Option<T>,
    max_denom: Option<T>,
}

impl<T> RationalGenerator<T> {
    /// Set the minimum value (inclusive).
    pub fn min_value(mut self, min_value: T) -> Self {
        self.min = Some(min_value);
        self
    }

    /// Set the maximum value (inclusive).
    pub fn max_value(mut self, max_value: T) -> Self {
        self.max = Some(max_value);
        self
    }

    /// Set the maximum allowed denominator (inclusive).
    pub fn max_denominator(mut self, max_denom: T) -> Self {
        self.max_denom = Some(max_denom);
        self
    }
}

impl<T: NumInteger + super::Integer + CheckedMul> RationalGenerator<T> {
    fn build_schema(&self) -> Value {
        let min = self.min.clone().unwrap_or_else(T::rational_default_min);
        let max = self.max.clone().unwrap_or_else(T::rational_default_max);
        let max_denom = self
            .max_denom
            .clone()
            .unwrap_or_else(T::rational_default_max);

        assert!(
            max_denom >= <T as super::Integer>::one(),
            "max_denominator must be >= 1"
        );
        assert!(
            max.checked_mul(&max_denom).is_some(),
            "max_value * max_denominator overflows the numerator type"
        );
        assert!(
            min.checked_mul(&max_denom).is_some(),
            "min_value * max_denominator overflows the numerator type"
        );

        cbor_map! {
            "type" => "rational",
            "min_value" => min.to_cbor(),
            "max_value" => max.to_cbor(),
            "max_denominator" => max_denom.to_cbor()
        }
    }
}

fn parse_ratio<T: super::Integer + NumInteger>(v: Value) -> Ratio<T> {
    let numer = T::from_cbor(map_get(&v, "numerator").cloned().unwrap());
    let denom = T::from_cbor(map_get(&v, "denominator").cloned().unwrap());
    Ratio::new(numer, denom)
}

impl<T> Generator<Ratio<T>> for RationalGenerator<T>
where
    T: super::Integer + NumInteger + CheckedMul,
{
    fn do_draw(&self, tc: &TestCase) -> Ratio<T> {
        parse_ratio::<T>(super::generate_raw(tc, &self.build_schema()))
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, Ratio<T>>> {
        Some(BasicGenerator::new(self.build_schema(), parse_ratio::<T>))
    }
}

/// Generate [`Ratio<T>`] values.
///
/// By default, uses `integers::<T>()` for the numerator and
/// `integers::<T>().min_value(T::one())` for the denominator.
/// Use `.numerator()` and `.denominator()` to customize.
///
/// # Examples
///
/// ```no_run
/// use num_rational::Ratio;
/// use hegel::generators::{self as gs, Generator};
///
/// #[hegel::test]
/// fn my_test(tc: hegel::TestCase) {
///     let r: Ratio<i64> = tc.draw(gs::rationals::<i64>());
///     assert!(*r.denom() > 0);
///
///     // Customize numerator and denominator ranges
///     let r: Ratio<i64> = tc.draw(gs::rationals::<i64>()
///         .numerator(gs::integers::<i64>().min_value(0).max_value(100))
///         .denominator(gs::integers::<i64>().min_value(1).max_value(10)));
///     assert!(*r.numer() >= 0 && *r.denom() >= 1);
/// }
/// ```
pub fn rationals<T: super::Integer>() -> RationalGenerator<T> {
    RationalGenerator {
        min: None,
        max: None,
        max_denom: None,
    }
}

// ---------------------------------------------------------------------------
// ComplexGenerator
// ---------------------------------------------------------------------------

/// Generator for [`Complex<T>`] values. Created by [`complex()`].
///
/// Draws the real and imaginary parts from separate generators.
pub struct ComplexGenerator<G> {
    real_gen: G,
    imag_gen: G,
}

impl<T, G> Generator<Complex<T>> for ComplexGenerator<G>
where
    G: Generator<T>,
    T: Clone + Num,
{
    fn do_draw(&self, tc: &TestCase) -> Complex<T> {
        let re = self.real_gen.do_draw(tc);
        let im = self.imag_gen.do_draw(tc);
        Complex::new(re, im)
    }
}

/// Generate [`Complex<T>`] values from the given real and imaginary generators.
///
/// # Example
///
/// ```no_run
/// use num_complex::Complex;
/// use hegel::generators::{self as gs, Generator};
///
/// #[hegel::test]
/// fn my_test(tc: hegel::TestCase) {
///     let c: Complex<f64> = tc.draw(gs::complex(
///         gs::floats::<f64>().min_value(-100.0).max_value(100.0),
///         gs::floats::<f64>().min_value(-100.0).max_value(100.0),
///     ));
///     assert!(c.re >= -100.0 && c.re <= 100.0);
/// }
/// ```
pub fn complex<T, G>(real_gen: G, imag_gen: G) -> ComplexGenerator<G>
where
    G: Generator<T>,
    T: Clone + Num,
{
    ComplexGenerator { real_gen, imag_gen }
}

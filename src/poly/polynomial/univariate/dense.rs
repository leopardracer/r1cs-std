use ark_ff::PrimeField;
use ark_relations::gr1cs::SynthesisError;
use ark_std::vec::Vec;

use crate::fields::{fp::FpVar, FieldVar};

/// Stores a polynomial in coefficient form, where coefficient is represented by
/// a list of `Fpvar<F>`.
pub struct DensePolynomialVar<F: PrimeField> {
    /// The coefficient of `x^i` is stored at location `i` in `self.coeffs`.
    pub coeffs: Vec<FpVar<F>>,
}

impl<F: PrimeField> DensePolynomialVar<F> {
    /// Constructs a new polynomial from a list of coefficients.
    pub fn from_coefficients_slice(coeffs: &[FpVar<F>]) -> Self {
        Self::from_coefficients_vec(coeffs.to_vec())
    }

    /// Constructs a new polynomial from a list of coefficients.
    pub fn from_coefficients_vec(coeffs: Vec<FpVar<F>>) -> Self {
        Self { coeffs }
    }

    /// Evaluates `self` at the given `point` and just gives you the gadget for
    /// the result. Caution for use in holographic lincheck: The output has
    /// 2 entries in one matrix
    pub fn evaluate(&self, point: &FpVar<F>) -> Result<FpVar<F>, SynthesisError> {
        // Horner's Method
        Ok(self
            .coeffs
            .iter()
            .rfold(FpVar::zero(), move |acc, coeff| acc * point + coeff))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        alloc::AllocVar, fields::fp::FpVar,
        poly::polynomial::univariate::dense::DensePolynomialVar, GR1CSVar,
    };
    use ark_poly::{polynomial::univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
    use ark_relations::gr1cs::ConstraintSystem;
    use ark_std::{test_rng, vec::Vec, UniformRand};
    use ark_test_curves::bls12_381::Fr;

    #[test]
    fn test_evaluate() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let cs = ConstraintSystem::new_ref();
            let poly: DensePolynomial<Fr> = DensePolynomial::rand(10, &mut rng);
            let poly_var = {
                let coeff: Vec<_> = poly
                    .coeffs
                    .iter()
                    .map(|&x| FpVar::new_witness(ns!(cs, "coeff"), || Ok(x)).unwrap())
                    .collect();
                DensePolynomialVar::from_coefficients_vec(coeff)
            };
            let point = Fr::rand(&mut rng);
            let point_var = FpVar::new_witness(ns!(cs, "point"), || Ok(point)).unwrap();

            let expected = poly.evaluate(&point);
            let actual = poly_var.evaluate(&point_var).unwrap();

            assert_eq!(actual.value().unwrap(), expected);
            assert!(cs.is_satisfied().unwrap());
        }
    }
}

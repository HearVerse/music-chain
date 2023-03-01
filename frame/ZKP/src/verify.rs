
use crate::verify::VerificationError::InvalidVerificationKey;
use bls12_381::{Bls12, G1Affine, G2Affine, Scalar};
use group::{prime::PrimeCurveAffine, Curve};
use pairing::{Engine, MultiMillerLoop};
use sp_std::{ops::AddAssign, prelude::*};

pub const SUPPORTED_CURVE: &str = "bls12381";
pub const SUPPORTED_PROTOCOL: &str = "groth16";

pub struct G1UncompressedBytes {
	inner: [u8; 96],
}

pub struct G2UncompressedBytes {
	inner: [u8; 192],
}

impl G1UncompressedBytes {
	pub fn new(x: [u8; 48], y: [u8; 48]) -> Self {
		let mut new_bytes: [u8; 96] = [0; 96];

		new_bytes[..48].copy_from_slice(&x[..48]);
		new_bytes[48..(48 + 48)].copy_from_slice(&y[..48]);
		new_bytes[0] |= &0u8;

		G1UncompressedBytes { inner: new_bytes }
	}
}

impl G2UncompressedBytes {
	pub fn new(x_c0: [u8; 48], x_c1: [u8; 48], y_c0: [u8; 48], y_c1: [u8; 48]) -> Self {
		let mut new_bytes: [u8; 192] = [0; 192];

		new_bytes[..48].copy_from_slice(&x_c1[..48]);
		new_bytes[48..(48 + 48)].copy_from_slice(&x_c0[..48]);
		new_bytes[96..(48 + 96)].copy_from_slice(&y_c1[..48]);
		new_bytes[144..(48 + 144)].copy_from_slice(&y_c0[..48]);

		new_bytes[0] |= &0u8;

		G2UncompressedBytes { inner: new_bytes }
	}
}

impl TryFrom<&G1UncompressedBytes> for G1Affine {
	type Error = ();

	fn try_from(value: &G1UncompressedBytes) -> Result<Self, Self::Error> {
		let g1 = G1Affine::from_uncompressed(&value.inner);
		if g1.is_none().into() {
			Err(())
		} else {
			Ok(g1.unwrap())
		}
	}
}

impl TryFrom<&G2UncompressedBytes> for G2Affine {
	type Error = ();

	fn try_from(value: &G2UncompressedBytes) -> Result<Self, Self::Error> {
		let g2 = G2Affine::from_uncompressed(&value.inner);
		if g2.is_none().into() {
			Err(())
		} else {
			Ok(g2.unwrap())
		}
	}
}

/// Represents Groth16 verification key
pub struct VerificationKey {
	pub alpha: G1Affine,
	pub beta: G2Affine,
	pub gamma: G2Affine,
	pub delta: G2Affine,
	pub ic: Vec<G1Affine>,
}

#[derive(Debug)]
pub enum VerificationKeyCreationError {
	PointCreationError,
}

impl VerificationKey {
	pub fn from_uncompressed(
		alpha: &G1UncompressedBytes,
		beta: &G2UncompressedBytes,
		gamma: &G2UncompressedBytes,
		delta: &G2UncompressedBytes,
		ic: &Vec<G1UncompressedBytes>,
	) -> Result<Self, VerificationKeyCreationError> {
		let alpha =
			alpha.try_into().map_err(|_| VerificationKeyCreationError::PointCreationError)?;
		let beta: G2Affine =
			beta.try_into().map_err(|_| VerificationKeyCreationError::PointCreationError)?;
		let gamma: G2Affine =
			gamma.try_into().map_err(|_| VerificationKeyCreationError::PointCreationError)?;
		let delta: G2Affine =
			delta.try_into().map_err(|_| VerificationKeyCreationError::PointCreationError)?;
		let mut ic_2: Vec<G1Affine> = Vec::with_capacity(ic.len());

		for i in ic {
			ic_2.push(
				G1Affine::try_from(i)
					.map_err(|_| VerificationKeyCreationError::PointCreationError)?,
			);
		}

		Ok(VerificationKey { alpha, beta, gamma, delta, ic: ic_2 })
	}
}

/// Represents Groth16 proof
pub struct GProof {
	pub a: G1Affine,
	pub b: G2Affine,
	pub c: G1Affine,
}

#[derive(Debug)]
pub enum GProofCreationError {
	PointCreationError,
}

impl GProof {
	pub fn from_uncompressed(
		a: &G1UncompressedBytes,
		b: &G2UncompressedBytes,
		c: &G1UncompressedBytes,
	) -> Result<Self, GProofCreationError> {
		let a = a.try_into().map_err(|_| GProofCreationError::PointCreationError)?;
		let b = b.try_into().map_err(|_| GProofCreationError::PointCreationError)?;
		let c = c.try_into().map_err(|_| GProofCreationError::PointCreationError)?;

		Ok(GProof { a, b, c })
	}
}

#[derive(Debug, PartialEq)]
pub enum VerificationError {
	InvalidVerificationKey,
}

pub type VerificationResult = Result<bool, VerificationError>;

pub type PublicInputs = Vec<Scalar>;

pub fn prepare_public_inputs(inputs: Vec<u64>) -> Vec<Scalar> {
	inputs.into_iter().map(Scalar::from).collect()
}

pub fn verify(vk: VerificationKey, proof: GProof, inputs: PublicInputs) -> VerificationResult {
	let public_inputs: &[<Bls12 as Engine>::Fr] = &inputs;

	if (public_inputs.len() + 1) != vk.ic.len() {
		return Err(InvalidVerificationKey)
	}

	let mut acc = vk.ic[0].to_curve();
	for (i, b) in public_inputs.iter().zip(vk.ic.iter().skip(1)) {
		AddAssign::<&<Bls12 as Engine>::G1>::add_assign(&mut acc, &(*b * i));
	}

	let a_b_pairing = Bls12::pairing(&proof.a, &proof.b);

	let final_result = Bls12::multi_miller_loop(&[
		(&vk.alpha, &vk.beta.into()),
		(&acc.to_affine(), &vk.gamma.into()),
		(&proof.c, &vk.delta.into()),
	])
	.final_exponentiation();

	Ok(a_b_pairing == final_result)
}
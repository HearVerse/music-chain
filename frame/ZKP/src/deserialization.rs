

use serde::{Deserialize, Deserializer};
use sp_std::vec::Vec;
use uint::construct_uint;

construct_uint! {
	pub struct U256(6);
}

type Number = [u8; 48];

type G1 = [Number; 3];
type G2 = [[Number; 2]; 3];

/// Struct representing snarkjs generated verification key
#[derive(Deserialize)]
pub struct VKey {
	#[serde(deserialize_with = "str_to_u8_vec_deserializer")]
	pub protocol: Vec<u8>,
	#[serde(deserialize_with = "str_to_u8_vec_deserializer")]
	pub curve: Vec<u8>,
	#[serde(alias = "nPublic")]
	pub public_inputs_len: u8,
	#[serde(alias = "vk_alpha_1")]
	#[serde(deserialize_with = "g1_deserializer")]
	pub alpha: G1,
	#[serde(alias = "vk_beta_2")]
	#[serde(deserialize_with = "g2_deserializer")]
	pub beta: G2,
	#[serde(alias = "vk_gamma_2")]
	#[serde(deserialize_with = "g2_deserializer")]
	pub gamma: G2,
	#[serde(alias = "vk_delta_2")]
	#[serde(deserialize_with = "g2_deserializer")]
	pub delta: G2,
	#[serde(alias = "IC")]
	#[serde(deserialize_with = "vec_g1_deserializer")]
	pub ic: Vec<G1>,
}

#[derive(Debug)]
pub enum VKeyDeserializationError {
	SerdeError,
}

impl VKey {
	/// Creates `VKey` from json representation
	pub fn from_json_u8_slice(slice: &[u8]) -> Result<Self, VKeyDeserializationError> {
		serde_json::from_slice(slice).map_err(|_| VKeyDeserializationError::SerdeError)
	}
}

/// Struct representing snarkjs generated proof
#[derive(Deserialize)]
pub struct Proof {
	#[serde(deserialize_with = "str_to_u8_vec_deserializer")]
	pub protocol: Vec<u8>,
	#[serde(deserialize_with = "str_to_u8_vec_deserializer")]
	pub curve: Vec<u8>,
	#[serde(alias = "pi_a")]
	#[serde(deserialize_with = "g1_deserializer")]
	pub a: G1,
	#[serde(alias = "pi_b")]
	#[serde(deserialize_with = "g2_deserializer")]
	pub b: G2,
	#[serde(alias = "pi_c")]
	#[serde(deserialize_with = "g1_deserializer")]
	pub c: G1,
}

#[derive(Debug)]
pub enum ProofDeserializationError {
	SerdeError,
}

impl Proof {
	/// Creates `Proof` from json representation
	pub fn from_json_u8_slice(slice: &[u8]) -> Result<Self, ProofDeserializationError> {
		serde_json::from_slice(slice).map_err(|_| ProofDeserializationError::SerdeError)
	}
}
/// Turns G1 point represented by numbers in decimal format into G1 point represented by numbers in
/// binary format
pub fn g1_deserializer<'de, D>(de: D) -> Result<[Number; 3], D::Error>
where
	D: Deserializer<'de>,
{
	let mut dec_numbers: [Number; 3] = [[0; 48]; 3];
	let s: [&str; 3] = serde::Deserialize::deserialize(de)?;
	for i in 0..3 {
		U256::from_dec_str(s[i]).unwrap().to_big_endian(dec_numbers[i].as_mut_slice());
	}
	Ok(dec_numbers)
}

/// Turns array of G1 points represented by numbers in decimal format into vector of G1 points
/// represented by numbers in binary format
pub fn vec_g1_deserializer<'de, D>(de: D) -> Result<Vec<[Number; 3]>, D::Error>
where
	D: Deserializer<'de>,
{
	let dec_numbers: Vec<[&str; 3]> = serde::Deserialize::deserialize(de)?;
	Ok(dec_numbers
		.iter()
		.map(|ic| {
			let mut arr: [Number; 3] = [[0; 48]; 3];
			for i in 0..3 {
				U256::from_dec_str(ic[i]).unwrap().to_big_endian(arr[i].as_mut_slice());
			}
			arr
		})
		.collect())
}

/// Turns G2 point represented by numbers in decimal format into G2 point represented by numbers in
/// binary format
pub fn g2_deserializer<'de, D>(de: D) -> Result<[[Number; 2]; 3], D::Error>
where
	D: Deserializer<'de>,
{
	let mut g2_numbers: [[Number; 2]; 3] = [[[0; 48]; 2]; 3];
	let dec_numbers: [[&str; 2]; 3] = serde::Deserialize::deserialize(de)?;
	for i in 0..3 {
		for j in 0..2 {
			U256::from_dec_str(dec_numbers[i][j])
				.unwrap()
				.to_big_endian(g2_numbers[i][j].as_mut_slice());
		}
	}
	Ok(g2_numbers)
}

/// Turns `str` into `Vec<u8>`
pub fn str_to_u8_vec_deserializer<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = serde::Deserialize::deserialize(de)?;
	Ok(s.as_bytes().into())
}

#[derive(Debug)]
pub enum PublicInputsDeserializationError {
	SerdeError,
}

/// Creates vector of `u64` representing public inputs
///
/// # Arguments
/// * `inputs` - A byte array slice containing array of integers in json array form
pub fn deserialize_public_inputs(
	inputs: &[u8],
) -> Result<Vec<u64>, PublicInputsDeserializationError> {
	let inputs: Vec<&str> = serde_json::from_slice(inputs).unwrap();
	let mut parsed_inputs: Vec<u64> = Vec::with_capacity(inputs.len());
	for input in inputs {
		match input.parse::<u64>() {
			Ok(n) => parsed_inputs.push(n),
			Err(_) => return Err(PublicInputsDeserializationError::SerdeError),
		}
	}
	Ok(parsed_inputs)
}

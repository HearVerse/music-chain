
pub use frame_support::inherent::Vec;

pub enum VerifierError{
    NoVerificationKey,
}

pub struct Verifier {
    pub key:Vec<u8>,
}

impl Verifier {

    pub fn verifier_proof(self,pub_input:u32,proof:Vec<u8>) -> Result<bool,VerifierError>{

        if self.key.is_empty(){
            return Err(VerifierError::NoVerificationKey)
        }
        // TODO implement Verifier Proof Codes
        Ok(proof.len() == pub_input as usize)
    }
}
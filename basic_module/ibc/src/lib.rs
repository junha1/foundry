/// All IBC light client module must export this to the IBC handler module
pub trait IbcLightClient {
    fn update(&self, header: &[u8]);

    fn latest_hight(&self) -> usize;

    /// TODO: decide whether to support prove-by-hash, instead of prove-by-value
    fn verify(&self, proof_height: usize, proof: Vec<u8>, value: Vec<u8>) -> Result<(), String>;
    fn verify_absence(&self, proof_height: usize, proof: Vec<u8>, value: Vec<u8>) -> Result<(), String>;
}

/// Client is responsible to encode 'foreign' structures
pub trait IbcEncoder {
    fn encode_connection_state(&self, counterparty_connection_identifier: &str, connection_end: ()) -> Vec<u8>;
    // and so on.
}

// Application layer data will be also encoded/verified by the light client.
// However, that is not required for all light client as it varies among the application.
// Those specified services are minimum requirements that all light client module must keep.

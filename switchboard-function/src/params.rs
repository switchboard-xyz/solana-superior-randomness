use crate::*;

pub struct ContainerParams {
    pub program_id: Pubkey,
    pub request_key: Pubkey,
}

impl ContainerParams {
    pub fn decode(container_params: &Vec<u8>) -> std::result::Result<Self, SbError> {
        let params = String::from_utf8(container_params.clone()).unwrap();

        let mut program_id: Pubkey = Pubkey::default();
        let mut request_key: Pubkey = Pubkey::default();


        println!("-----> {:?}", params);
        for env_pair in params.split(',') {
            let pair: Vec<&str> = env_pair.splitn(2, '=').collect();
            if pair.len() == 2 {
                match pair[0] {
                    "PID" => program_id = Pubkey::from_str(pair[1]).unwrap(),
                    "REQUEST_KEY" => request_key = Pubkey::from_str(pair[1]).unwrap(),
                    _ => {}
                }
            }
        }

        if program_id == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "PID cannot be undefined".to_string(),
            ));
        }
        if request_key == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "REQUEST_KEY cannot be undefined".to_string(),
            ));
        }

        Ok(Self {
            program_id,
            request_key
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_decode() {
        let request_params_string = format!(
            "PID={},REQUEST={}",
            anchor_spl::token::ID,
            anchor_spl::token::ID
        );
        let request_params_bytes = request_params_string.into_bytes();

        let params = ContainerParams::decode(&request_params_bytes).unwrap();

        assert_eq!(params.program_id, anchor_spl::token::ID);
        assert_eq!(params.request_key, anchor_spl::token::ID);
    }
}

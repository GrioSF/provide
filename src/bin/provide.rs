use rusoto_core::{HttpClient, Region};
use rusoto_core::credential::EnvironmentProvider;
use rusoto_ssm::*;
use std::error::Error;

// aws ssm get-parameters-by-path --region us-west-1 --with-decryption --path /moment/staging
fn main() {
    let path = String::from("/moment/staging");
    match get_parameters(path, None) {
        Ok(params) => println!("{:?}", params),
        Err(err) => println!("{:?}", err),
    };
}

fn get_parameters(path: String, next_token: Option<String>) -> Result<Box<Vec<Parameter>>, Box<Error>> {
    fn get_parameters_with_acc(path: String, next_token: Option<String>, mut acc: Box<Vec<Parameter>>) -> Result<Box<Vec<Parameter>>, Box<Error>> {
        let region = Region::UsWest1;
        let request_dispatcher = HttpClient::new().unwrap();
        let credentials_provider = EnvironmentProvider::default();
        let ssm_client = SsmClient::new_with(request_dispatcher, credentials_provider, region);
        let input = GetParametersByPathRequest{
            path: path.clone(),
            next_token: next_token,
            recursive: Some(false),
            with_decryption: Some(false),
            ..Default::default()
        };
        match ssm_client.get_parameters_by_path(input).sync() {
            Ok(output) => {
                acc.append(&mut output.parameters.unwrap());
                match output.next_token {
                    Some(token) => {
                        get_parameters_with_acc(path.clone(), Some(token), acc)
                    },
                    None => Ok(acc)
                }
            },
            Err(err) => Err(Box::new(err)),
        }
    }
    let seed = Vec::<Parameter>::new();
    get_parameters_with_acc(path, next_token, Box::new(seed))
}
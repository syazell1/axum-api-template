use unicode_segmentation::UnicodeSegmentation;
use validator::{Validate, ValidationError};
use crate::errors::AppError;
use crate::features::auth::models::{LoginFormData, RegisterFormData};

#[derive(Validate)]
pub struct Credentials {
    #[validate(custom(function = "parse_username"))]
    pub username : String,
    #[validate(custom(function = "parse_password"))]
    pub password : String
}

fn parse_username (v : &str) -> Result<(), ValidationError>{ 
    let is_empty = v.trim().is_empty();

    let is_too_long = v.graphemes(true).count() > 12;

    if is_empty || is_too_long {
        return Err(ValidationError::new("invalid_username").with_message(std::borrow::Cow::Borrowed("Invalid Username")))
    }

    Ok(())
}

fn parse_password (v : &str) -> Result<(), ValidationError>{ 
    let is_empty = v.trim().is_empty();

    let is_too_long = v.graphemes(true).count() > 12;

    if is_empty || is_too_long {
        return Err(ValidationError::new("invalid_username").with_message(std::borrow::Cow::Borrowed("Invalid Username")))
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::Fake;
    use validator::Validate;

    use crate::utils::randomizer::generate_random_string;

    use super::Credentials;
 
    fn generate_test_user() -> Credentials {
        Credentials {
            username : (1..12).fake(),
            password : (1..12).fake()
        }
    }
    #[test]
    fn a_valid_credential_is_accepted() {
        let credentials = generate_test_user();

        let result = credentials.validate();

        assert_ok!(result);
    } 

    #[test]
    fn an_empty_username_and_password_is_rejected() {
        for v in 0..1 {
            let mut credentials = generate_test_user();
            if v==0  {
                credentials.username = "".into();
                let result = credentials.validate();

                assert_err!(result);
            }

            if v==1 {
                credentials.password = "".into();
                let result = credentials.validate();

                assert_err!(result);
            }
        }
    }

    #[test]
    fn a_long_username_is_rejected() {
        let mut credentials = generate_test_user();
        let test_username = vec![generate_random_string(14), "انفسكم".repeat(12).into()];

        for v in test_username.iter() {
            credentials.username = v.to_string();

            let result = credentials.validate();
            assert_err!(result);
        }
    }


    #[test]
    fn a_long_password_is_rejected() {
        let mut credentials = generate_test_user();
        let test_password= vec![generate_random_string(14), "انفسكم".repeat(12).into()];

        for v in test_password.iter() {
            credentials.password = v.to_string();

            let result = credentials.validate();
            assert_err!(result);
        }
    }
}

impl TryFrom<LoginFormData> for Credentials {
    type Error = AppError;

    fn try_from(value: LoginFormData) -> Result<Self, Self::Error> {
        let LoginFormData {username, password} = value;

        let credentials = Credentials {username, password};

        credentials.validate()?;

        Ok(credentials)
    }
}

impl TryFrom<RegisterFormData> for Credentials {
    type Error = AppError;

    fn try_from(value: RegisterFormData) -> Result<Self, Self::Error> {
        let RegisterFormData{username, password} = value;

        let credentials = Credentials {username, password};

        credentials.validate()?;

        Ok(credentials)
    }
}
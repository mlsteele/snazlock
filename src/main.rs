extern crate pam_auth;
pub fn main() {
        let service = "<yourapp>";
        let user = "<user>";
        let password = "<pass>";

        let mut auth = pam_auth::Authenticator::new(service).unwrap();
        auth.set_credentials(user, password);
        if auth.authenticate().is_ok() && auth.open_session().is_ok() {
            println!("Successfully opened a session!");
        }
        else {
            println!("Authentication failed =/");
        }
}

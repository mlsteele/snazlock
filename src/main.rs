extern crate pam_auth;
extern crate rpassword;
extern crate users;


const PAM_APP_NAME: &'static str = "snazlock";

pub fn main() {
    let username = users::get_user_by_uid(users::get_current_uid()).unwrap().name().to_owned();
    println!("Hello, {}!", username);

    println!("password me up >");
    let password = rpassword::read_password().unwrap();
    println!("got it.");

    let mut auth = pam_auth::Authenticator::new(PAM_APP_NAME).unwrap();
    auth.set_credentials(&username, &password);
    if auth.authenticate().is_ok() && auth.open_session().is_ok() {
        println!("Successfully opened a session!");
    }
    else {
        println!("Authentication failed =/");
    }
}

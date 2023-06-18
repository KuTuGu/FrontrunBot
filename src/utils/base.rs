use std::env;

pub fn get_env(name: &str) -> String {
    env::var(name).expect(&format!("Expect environment variable <{}>", name))
}

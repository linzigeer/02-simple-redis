use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Locked;
#[derive(Debug)]
pub struct Unlocked;

#[derive(Debug)]
pub struct PasswordManager<Status> {
    user_id: u32,
    password: HashMap<String, String>,
    status: PhantomData<Status>,
}

impl PasswordManager<Locked> {
    pub fn unlock(self) -> PasswordManager<Unlocked> {
        PasswordManager {
            user_id: self.user_id,
            password: self.password,
            status: PhantomData::<Unlocked>,
        }
    }
}

impl PasswordManager<Unlocked> {
    pub fn lock(self) -> PasswordManager<Locked> {
        PasswordManager {
            user_id: self.user_id,
            password: self.password,
            status: PhantomData::<Locked>,
        }
    }

    pub fn update_password(&mut self, user_name: String, new_password: String) {
        self.password.entry(user_name).or_insert(new_password);
    }
}

impl<Status: Debug> PasswordManager<Status> {
    pub fn print(&self) {
        println!("{self:?}");
    }
}

impl<Status> PasswordManager<Status> {
    pub fn new(user_id: u32) -> Self {
        Self {
            user_id,
            password: Default::default(),
            status: Default::default(),
        }
    }
}

pub fn main() {
    println!("hello  world1");
}

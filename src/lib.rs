extern crate regex;

use std::sync::{Arc,Mutex,Once,ONCE_INIT};
use std::{mem};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use regex::{Regex};

#[derive(Clone)]
struct QueryRegistry {
    inner: Arc<Mutex<HashMap<String,String>>>
}

fn registry() -> QueryRegistry {
    static mut PRESQL_REGISTRY: *const QueryRegistry = 0 as *const QueryRegistry;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            let reg = QueryRegistry {
                inner: Arc::new(Mutex::new(HashMap::new()))
            };

            PRESQL_REGISTRY = mem::transmute(Box::new(reg));
        });

        (*PRESQL_REGISTRY).clone()
    }
}

impl QueryRegistry {
    fn get(key: String) -> String{
        let r = registry();
        let map = r.inner.lock().unwrap();
        let opt = map.get(&key);
        match opt{
             Some(s) => {(*s).clone() }
             _ => panic!("Request `{}` not found.", key)
        }
    }

    fn set(key: String, value: String){
        let r = registry();
        let mut data = r.inner.lock().unwrap();
        data.insert(key, value);
    }

    fn register(file_name: &str, alias: &str){
        let mut f = match File::open(file_name){
             Err(_) => { panic!("Unable to open {}", file_name); },
             Ok(f) => {f},
        };
        let mut s = String::new();
        match f.read_to_string(&mut s){
            Err(_) => {
                panic!(
                    format!("couldn't read {}", file_name));
            },
            Ok(_) => {},
        };
        let regex_str = r"(?is)--\s*name:\s*(?P<key>\w+)\s*(?P<query>.+?);\n";
        let regex = Regex::new(regex_str).unwrap();
        for caps in regex.captures_iter(&(*s)){
            let key = alias.to_string();
            let key = key + "/";
            QueryRegistry::set(key + caps.name("key").unwrap(),
                               caps.name("query").unwrap().to_string());
        }
    }
}
#[]
macro_rules! presql{
    ($alias: expr, $query: expr) => {{$crate::get( $alias , $query )}}
}

// PUBLIC API

pub fn get(alias: &'static str, query: &'static str) -> String{
    QueryRegistry::get(format!("{}/{}", alias, query))
}

pub fn register(file_name: &str, alias: &str){
    QueryRegistry::register(file_name, alias)
}

#[test]
fn test_get_set() {
    QueryRegistry::set("i/i".to_string(), "I".to_string());
    assert_eq!(QueryRegistry::get("i/i".to_string()), "I".to_string());
    assert_eq!(get("i", "i"), "I".to_string());
}

#[test]
fn test_register() {
    QueryRegistry::register("src/tests.sql", "test_alias");
    assert_eq!(get("test_alias", "test_query1"),
               "SELECT * FROM test_table".to_string());
    assert_eq!(QueryRegistry::get("test_alias/test_query2".to_string()),
               "SELECT * FROM test_table2".to_string());
}

#[test]
fn test_macro(){
    QueryRegistry::register("src/tests.sql", "test_alias");
    assert_eq!(presql!("test_alias", "test_query1"),
               "SELECT * FROM test_table".to_string());
}

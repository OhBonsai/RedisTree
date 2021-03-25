#[macro_use]
extern crate redis_module;

use redis_module::native_types::RedisType;
use redis_module::{raw, Context, NextArg, RedisResult, RedisValue, RedisString, REDIS_OK};
use redis_module::logging::{log as redis_log};
use redis_module::LogLevel;
use std::os::raw::{c_void, c_int, c_char};
use std::ptr;
use std::ffi::{CStr, CString};
use trees::*;
use std::convert::TryFrom;


// =================================================================================================
// LOG
// =================================================================================================
fn log(message: &str) {
    let mut info = "tree: ".to_string();
    info.push_str(message);
    redis_log(LogLevel::Warning, &info)
}


#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error { msg: e }
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error { msg: e.to_string() }
    }
}

impl From<Error> for redis_module::RedisError {
    fn from(e: Error) -> Self {
        redis_module::RedisError::String(e.msg)
    }
}


use std::collections::HashMap;

#[derive(Debug)]
struct RedisTreeType {
    data: Tree<String>,
    // map:  HashMap<String, String>
}

impl RedisTreeType {
    fn to_string(&self) -> String {
        self.data.to_string()
    }
}


#[allow(non_snake_case, unused)]
pub extern "C" fn init(_: *mut raw::RedisModuleCtx) -> c_int {
    raw::Status::Ok as c_int
}

#[allow(non_snake_case, unused)]
pub unsafe extern "C" fn rdb_load(rdb: *mut raw::RedisModuleIO, encver: c_int) -> *mut c_void {
    if let Ok(tree) = Tree::try_from(raw::load_string(rdb)) {
        Box::into_raw(Box::new(tree)) as *mut c_void
    } else {
        Box::into_raw(Box::new(Tree::new("rdb_load_fail"))) as *mut c_void
    }

}

#[allow(non_snake_case, unused)]
pub unsafe extern "C" fn rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    let tree = (&*(value as *mut Tree<String>)).to_string();
    raw::save_string(rdb, tree.as_str());


    // let tree = &*(value as *mut Tree<String>);
    // let tree_string = tree.to_string();
    // let c_str = CString::new(tree_string.as_str()).unwrap();
    // raw::RedisModule_SaveStringBuffer.unwrap()(rdb, c_str.as_ptr() as *const c_char, tree_string.len());
}


#[allow(non_snake_case, unused)]
pub unsafe extern "C" fn aof_rewrite(aof: *mut raw::RedisModuleIO, key: *mut raw::RedisModuleString, value: *mut c_void) {
    // do nothing
}



#[allow(non_snake_case, unused)]
pub unsafe extern "C" fn free(value: *mut c_void) {
    Box::from_raw(value as *mut RedisTreeType);
}


#[allow(non_snake_case, unused)]
pub unsafe extern "C" fn aux_load(rdb: *mut raw::RedisModuleIO, encver: i32, when: i32) -> i32 {
    raw::Status::Ok as i32
}

#[allow(non_snake_case, unused)]
pub unsafe extern "C" fn aux_save(rdb: *mut raw::RedisModuleIO, when: i32) {
}


static TREE_TYPE: RedisType = RedisType::new(
    "ReTreeYou",
    0,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: Some(rdb_load),
        rdb_save: Some(rdb_save),
        aof_rewrite: None,
        free: Some(free),
        mem_usage: None,
        digest: None,
        aux_load: None,
        aux_save: None,
        aux_save_triggers: 0,
    },
)execution failed error during connect: Get http://%2Fvar%2Frun%2Fdocker.sock/v1.40/containers/he6j1i859tsdcm6s9ukcg8muicl3dxib/json: context canceled        {"stage-d": 9095, "rep;



fn init_tree(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key_writable(&args.next_string()?);

    key.set_value(&TREE_TYPE, Tree::try_from(args.next_string()?)?)?;
    REDIS_OK
}

fn get_tree(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key(&args.next_string()?);

    let value = match key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        Some(value) => value.to_string().into(),
        None => RedisValue::Null,
    };

    Ok(value)
}

fn get_subtree(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key(&args.next_string()?);
    let node_data = args.next_string()?;

    if let Some(value) = key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        if let  Some(node) = value.data.root().locate_first_by_data(&node_data) {
            return Ok(node.to_string().into())
        }
    }
    Ok(RedisValue::Null)
}


fn del_tree(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key_writable(&args.next_string()?);

    match key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        Some(_) => {
            key.delete()?;
            REDIS_OK
        }
        None => Ok(RedisValue::Null),
    }
}

fn del_subtree(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let mut key = ctx.open_key_writable(&args.next_string()?);
    let node_data = args.next_string()?;


    if let Some(mut value) = key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        if let  Some(mut node) = value.data.root_mut().locate_first_mut_by_data(&node_data) {
            return Ok(node.detach().to_string().into())
        }
    }
    Ok(RedisValue::Null)
}

fn set_tail_child(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let mut key = ctx.open_key_writable(&args.next_string()?);
    let node_data = args.next_string()?;
    // let path = args.next_string()?.split(".").map(|v| v.to_string()).collect::<Vec<String>>();
    let sub_tree = Tree::try_from(args.next_string()?)?;


    if let Some(mut value) = key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        if let Some(mut node) = value.data.root_mut().locate_first_mut_by_data(&node_data) {
            node.push_back(sub_tree);
            return REDIS_OK;
        }
    }

    Ok(RedisValue::Null)
}


fn get_ancestors(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key(&args.next_string()?);
    let node_data = args.next_string()?;

    if let Some(value) = key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        if let Some(node) = value.data.root().locate_first_by_data(&node_data) {
            let ancestors = node.ancestors();
            if ancestors.len() > 0 {
                return Ok(RedisValue::Array(ancestors.into_iter().map(|v|{
                    v.clone().into()
                }).collect::<Vec<_>>()))
            }
        }
    }

    Ok(RedisValue::Null)
}


fn get_descendants(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key(&args.next_string()?);
    let node_data = args.next_string()?;

    if let Some(value) = key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        if let Some(node) = value.data.root().locate_first_by_data(&node_data) {
            let descendants = node.descendants();
            if descendants.len() > 0 {
                return Ok(RedisValue::Array(descendants.into_iter().map(|v|{
                    v.clone().into()
                }).collect::<Vec<_>>()))
            }
        }
    }

    Ok(RedisValue::Null)
}


fn get_father(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key(&args.next_string()?);
    let node_data = args.next_string()?;

    if let Some(value) = key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        if let Some(node) = value.data.root().locate_first_by_data(&node_data) {
            if let Some(father) = node.father() {
                return Ok(father.into());
            } 
        }
    }

    Ok(RedisValue::Null)
}


fn get_children(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = ctx.open_key(&args.next_string()?);
    let node_data = args.next_string()?;

    if let Some(value) = key.get_value::<RedisTreeType>(&TREE_TYPE)? {
        if let Some(node) = value.data.locate_first_by_data(&node_data) {
            let children = node.children();
            if children.len() > 0 {
                return Ok(RedisValue::Array(children.into_iter().map(|v|{
                    v.clone().into()
                }).collect::<Vec<_>>()))
            }
        }
    }

    Ok(RedisValue::Null)
}



redis_module! {
    name: "ReTree",
    version: 1,
    data_types: [
        TREE_TYPE,
    ],
    init: init,
    commands: [
        ["tree.init", init_tree, "write", 1, 1, 1],
        ["tree.get", get_tree, "readonly", 1, 1, 1],
        ["tree.del", del_tree, "write", 1, 1, 1],

        ["tree.get_subtree", get_subtree, "readonly", 1, 1, 1],
        ["tree.del_subtree", del_subtree, "write", 1, 1, 1],
        ["tree.set_subtree", set_tail_child, "write", 1, 1, 1],
        ["tree.get_ancestors", get_ancestors, "readonly", 1, 1, 1],
        ["tree.get_descendants", get_descendants, "readonly", 1, 1, 1],
        ["tree.get_father", get_father, "readonly", 1, 1, 1],
        ["tree.get_children", get_children, "readonly", 1, 1, 1],
    ],
}
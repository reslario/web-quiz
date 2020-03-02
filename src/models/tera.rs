// This file contains mostly workarounds required because rocket uses an old version of tera
// which doesn't have some of the features used in this project.
// At the time of writing this, there's already a commit bumping the tera dependency's version,
// but it's not included in any library releases yet.
// Once it is, though, this code can probably be deleted.

use {
    std::collections::HashMap,
    rocket_contrib::templates::tera::{
        Tera,
        Value,
        Result,
        to_value,
        try_get_value
    }
};

pub fn configure(tera: &mut Tera) {
    tera.register_filter("map", map)
}

// mostly copied from https://github.com/Keats/tera/blob/master/src/builtins/filters/array.rs
pub fn map(value: Value, args: HashMap<String, Value>) -> Result<Value> {
    let arr = try_get_value!("map", "value", Vec<Value>, value);
    if arr.is_empty() {
        return Ok(arr.into());
    }

    let attribute = match args.get("attribute") {
        Some(val) => try_get_value!("map", "attribute", String, val),
        None => return Err("The `map` filter has to have an `attribute` argument".into()),
    };

    let json_pointer = get_json_pointer(&attribute);
    let arr = arr
        .into_iter()
        .filter_map(|v| match v.pointer(&json_pointer) {
            Some(val) if !val.is_null() => Some(val.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(to_value(arr).unwrap())
}

// copied from https://github.com/Keats/tera/blob/master/src/context.rs
// since it's private there
pub fn get_json_pointer(key: &str) -> String {
    ["/", &key.replace(".", "/")].join("")
}
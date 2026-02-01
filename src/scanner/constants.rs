//! Constants and predicates for engine/GUT callbacks.

/// Godot engine callbacks / virtual methods – always considered "used"
const ENGINE_CALLBACKS: &[&str] = &[
    "_init",
    "_ready",
    "_enter_tree",
    "_exit_tree",
    "_process",
    "_physics_process",
    "_input",
    "_gui_input",
    "_unhandled_input",
    "_unhandled_key_input",
    "_draw",
    "_notification",
    "_get",
    "_set",
    "_get_property_list",
    "_validate_property",
    "_to_string",
];

/// GUT (Godot Unit Test) lifecycle hooks – framework calls these; treat as used
const GUT_HOOKS: &[&str] = &[
    "before_each",
    "after_each",
    "before_all",
    "after_all",
    "before_test",
    "after_test",
];

pub fn is_engine_callback(name: &str) -> bool {
    ENGINE_CALLBACKS.contains(&name)
}

/// True if name is a GUT test method (func test_*) or GUT hook which the framework runs (case-insensitive for test_ prefix).
pub fn is_gut_test_function(name: &str) -> bool {
    name.len() >= 5
        && name[..5].eq_ignore_ascii_case("test_")
        || GUT_HOOKS.contains(&name)
}

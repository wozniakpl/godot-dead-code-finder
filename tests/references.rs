//! Integration tests for find_function_references and find_tscn_references.

use std::path::Path;

use gdcf::scanner::{find_function_references, find_tscn_references};

#[test]
fn find_function_references_direct_call() {
    let source = r#"
func _ready():
    do_thing()

func do_thing():
    pass
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"do_thing"));
    assert!(names.contains(&"_ready"));
}

#[test]
fn find_function_references_connect() {
    let source = r#"
func _ready():
    $Button.pressed.connect(_on_button_pressed)

func _on_button_pressed():
    pass
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"_on_button_pressed"));
}

#[test]
fn find_function_references_call_string() {
    let source = r#"
func _ready():
    call("dynamic_method")
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"dynamic_method"));
}

#[test]
fn find_function_references_nested_call() {
    let source = r#"
func _load_settings() -> void:
    for p in players:
        player.volume_db = _linear_to_db(_get_effective_music_volume())

func _linear_to_db(linear: float) -> float:
    return linear

func _get_effective_music_volume() -> float:
    return music_volume * global_volume
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(
        names.contains(&"_get_effective_music_volume"),
        "nested call inner(outer()) should count as reference"
    );
    assert!(names.contains(&"_linear_to_db"));
}

#[test]
fn find_function_references_tween_method_callback() {
    let source = r#"
const TWEEN_FADE_AUDIO_DURATION = 0.5

func set_master_volume(volume_db: float) -> void:
    master_volume = volume_db
    master_volume_changed.emit(master_volume)

func transition_master_volume(from_volume: float, to_volume: float) -> void:
    if _fade_tween != null:
        _fade_tween.kill()
    _fade_tween = create_tween()
    _fade_tween.tween_method(set_master_volume, from_volume, to_volume, TWEEN_FADE_AUDIO_DURATION)
"#;
    let refs = find_function_references(Path::new("audio.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(
        names.contains(&"set_master_volume"),
        "tween_method(set_master_volume, ...) should count as reference"
    );
}

#[test]
fn find_function_references_assigned_to_dict() {
    let source = r#"
func _ready() -> void:
    context["print"] = _console_print

func _console_print(arg) -> void:
    output.append_text(str(arg) + "\n")
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(
        names.contains(&"_console_print"),
        "dict[key] = func should count as reference"
    );
}

#[test]
fn test_find_tscn_references() {
    let source =
        r#"[connection signal="pressed" from="Button" to="." method="_on_quit_dialog_confirmed"]"#;
    let refs = find_tscn_references(Path::new("ui.tscn"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"_on_quit_dialog_confirmed"));
    assert_eq!(refs.len(), 1);
}

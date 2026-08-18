#!/usr/bin/env python3
"""Turn unproven animation write-back into explicit read-only behavior."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / "apps/desktop/src-tauri/src/commands/animation.rs"
text = PATH.read_text(encoding="utf-8")
text = text.replace(
    "    Animation, AnimationCategory, AnimationFrame, AnimationLoader, AnimationWriter,\n",
    "    Animation, AnimationCategory, AnimationFrame, AnimationLoader,\n",
)
replacement = '''fn mutate_animation<F>(
    _state: &AppState,
    _boxer_key: &str,
    _animation_name: &str,
    _mutate_fn: F,
) -> Result<(), String>
where
    F: FnOnce(&mut Animation) -> Result<(), String>,
{
    Err(
        "Animation, frame, hitbox, and hurtbox mutation is research-blocked: ROM write-back is not sufficiently reverse-engineered to guarantee persistence. The animation surface is read-only until round-trip write tests pass."
            .to_string(),
    )
}
'''
text, count = re.subn(
    r'fn mutate_animation<F>\(.*?\n}\n\n// ============================================================================\n// READ COMMANDS',
    replacement + '\n// ============================================================================\n// READ COMMANDS',
    text,
    count=1,
    flags=re.S,
)
if count == 0 and "Animation, frame, hitbox, and hurtbox mutation is research-blocked" not in text:
    raise SystemExit("animation mutation helper source shape changed")
PATH.write_text(text, encoding="utf-8")
print("Animation mutation is explicitly read-only/research-blocked.")
